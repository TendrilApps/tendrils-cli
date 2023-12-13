use std::path::{Path, PathBuf};
mod tendril;
use tendril::Tendril;

// TODO: Recursively look through all parent folders
// TODO: If it can't be found in the current path, check in an env variable 
pub fn get_tendrils_folder(starting_path: &Path) -> Option<PathBuf> {
    if is_tendrils_folder(&starting_path) {
        Some(starting_path.to_owned())
    } else {
        None
    }
}

pub fn get_tendrils(tendrils_folder: &Path) -> Result<Vec<Tendril>, GetTendrilsError> {
    let tendrils_file_path = Path::new(&tendrils_folder).join("tendrils.json");
    let tendrils_file_contents = std::fs::read_to_string(&tendrils_file_path)?;
    let tendrils = parse_tendrils(&tendrils_file_contents)?;
    Ok(tendrils)
}

pub fn get_tendril_overrides(tendrils_folder: &Path) -> Result<Vec<Tendril>, GetTendrilsError> {
    let tendrils_file_path = Path::new(&tendrils_folder).join("tendrils-override.json");
    let tendrils_file_contents: String;

    if tendrils_file_path.is_file() {
        tendrils_file_contents = std::fs::read_to_string(&tendrils_file_path)?;
    } else {
        return Ok([].to_vec())
    }

    let tendrils = parse_tendrils(&tendrils_file_contents)?;
    Ok(tendrils)
}

pub fn is_tendrils_folder(dir: &Path) -> bool {
    dir.join("tendrils.json").is_file()
}

/// # Arguments
/// - `json` - JSON array of Tendrils
fn parse_tendrils(json: &str) -> Result<Vec<Tendril>, serde_json::Error> {
    serde_json::from_str::<Vec<Tendril>>(&json)
}

/// Returns a list of all Tendrils after replacing global ones with any
/// applicable overrides.
/// # Arguments
/// - `global` - The set of Tendrils (typically defined in tendrils.json)
/// - `overrides` - The set of Tendril overrides (typically defined in tendrils-overrides.json)
pub fn resolve_overrides(global: &Vec<Tendril>, overrides: &Vec<Tendril>) -> Vec<Tendril> {
    let mut resolved_tendrils: Vec<Tendril> = Vec::from([]);

    for tendril in global {
        let mut last_index: usize = 0;
        let overrides_iter = overrides.into_iter();

        if overrides_iter.enumerate().any(|(i, x)| { 
            last_index = i;
            x.id() == tendril.id() })
        {
            resolved_tendrils.push(overrides[last_index].clone());
        }
        else {
            resolved_tendrils.push(tendril.clone())
        }
    }

    resolved_tendrils
}

#[derive(Debug)]
pub enum GetTendrilsError {
    IoError(std::io::Error),
    ParseError(serde_json::Error)
}

impl From<std::io::Error> for GetTendrilsError {
    fn from(err: std::io::Error) -> Self {
        GetTendrilsError::IoError(err)
    }
}

impl From<serde_json::Error> for GetTendrilsError {
    fn from(err: serde_json::Error) -> Self {
        GetTendrilsError::ParseError(err)
    }
}

#[cfg(test)]
mod sample_tendrils;
#[cfg(test)]
use sample_tendrils::SampleTendrils;
#[cfg(test)]
extern crate tempdir;

#[cfg(test)]
fn get_disposable_folder() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("temp-tendrils-folders");
    
    if !path.is_dir() {
        std::fs::create_dir(&path).unwrap();
    }
    path
}

#[cfg(test)]
mod is_tendrils_folder_tests {
    use serial_test::serial;
    use super::is_tendrils_folder;
    use super::get_disposable_folder;
    use super::tempdir::TempDir;

    #[test]
    #[serial] // To avoid flaky tests upon first run (before the disposable folder exists)
    fn empty_dir_returns_false() {
        let temp = TempDir::new_in(
            get_disposable_folder(),
            "Empty").unwrap();

        assert!(!is_tendrils_folder(&temp.path()));
    }

    #[test]
    #[serial]
    fn misc_other_files_only_returns_false() {
        let temp = TempDir::new_in(
            get_disposable_folder(),
            "MiscOtherFiles").unwrap();
        std::fs::File::create(temp.path().join("misc.txt")).unwrap();

        assert!(!is_tendrils_folder(&temp.path()));
    }

    #[test]
    #[serial]
    fn has_tendrils_json_dir_returns_false() {
        let temp = TempDir::new_in(
            get_disposable_folder(),
            "TendrilsJsonSubdir").unwrap();
        std::fs::create_dir(temp.path().join("tendrils.json")).unwrap();

        assert!(!is_tendrils_folder(&temp.path()));
    }

    #[test]
    #[serial]
    fn has_tendrils_json_file_returns_true() {
        let temp = TempDir::new_in(
            get_disposable_folder(),
            "EmptyTendrilsJson").unwrap();
        std::fs::File::create(temp.path().join("tendrils.json")).unwrap();

        assert!(is_tendrils_folder(&temp.path()));
    }
}

#[cfg(test)]
mod get_tendrils_folder_tests {
    use serial_test::serial;
    use super::get_disposable_folder;
    use super::get_tendrils_folder;
    use super::tempdir::TempDir;

    #[test]
    #[serial]
    fn starting_dir_not_tendrils_folder_returns_none() {
        let temp = TempDir::new_in(
            get_disposable_folder(),
            "Empty").unwrap();

        let actual = get_tendrils_folder(&temp.path());

        assert!(actual.is_none());
    }

    #[test]
    #[serial]
    fn starting_dir_is_tendrils_folder_returns_starting_dir() {
        let temp = TempDir::new_in(
            get_disposable_folder(),
            "EmptyTendrilsJson").unwrap();
        std::fs::File::create(temp.path().join("tendrils.json")).unwrap();

        let actual = get_tendrils_folder(&temp.path()).unwrap();

        assert_eq!(actual, temp.path());
    }
}

#[cfg(test)]
mod get_tendrils_tests {
    use std::matches;
    use serial_test::serial;
    use super::{ get_tendrils, get_disposable_folder };
    use super::{ GetTendrilsError, SampleTendrils, tempdir::TempDir };

    #[test]
    #[serial]
    fn no_tendrils_json_file_returns_err() {
        let temp = TempDir::new_in(
            get_disposable_folder(),
            "Empty").unwrap();

        let actual  = get_tendrils(&temp.path());

        assert!(matches!(actual.unwrap_err(), GetTendrilsError::IoError(_)));
    }

    #[test]
    #[serial]
    fn invalid_json_returns_err() {
        let tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "InvalidTendrilsJson").unwrap();
        let tendrils_json = &tendrils_folder.path().join("tendrils.json");
        std::fs::File::create(&tendrils_json).unwrap();
        std::fs::write(&tendrils_json, "I'm not JSON").unwrap();

        let actual = get_tendrils(&tendrils_folder.path());

        assert!(matches!(actual.unwrap_err(), GetTendrilsError::ParseError(_)));
    }

    #[test]
    #[serial]
    fn valid_json_returns_tendrils() {
        let tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "ValidJson").unwrap();
        let samples = SampleTendrils::new();
        let json = SampleTendrils::build_tendrils_json(
            &[samples.tendril_1_json].to_vec());
        let tendrils_json = &tendrils_folder.path().join("tendrils.json");
        std::fs::File::create(&tendrils_json).unwrap();
        std::fs::write(&tendrils_json, &json).unwrap();

        let expected = [samples.tendril_1].to_vec();

        let actual  = get_tendrils(&tendrils_folder.path()).unwrap();

        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod get_tendril_overrides_tests {
    use std::matches;
    use serial_test::serial;

    use super::{ get_tendril_overrides, get_disposable_folder };
    use super::{ GetTendrilsError, SampleTendrils, tempdir::TempDir };

    #[test]
    #[serial]
    fn no_tendrils_json_file_returns_empty() {
        let temp = TempDir::new_in(
            get_disposable_folder(),
            "Empty").unwrap();

        let actual  = get_tendril_overrides(&temp.path()).unwrap();

        assert!(actual.is_empty())
    }

    #[test]
    #[serial]
    fn invalid_json_returns_err() {
        let tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "InvalidTendrilsOverridesJson").unwrap();
        let tendrils_override_json = &tendrils_folder.path().join("tendrils-override.json");
        std::fs::File::create(&tendrils_override_json).unwrap();
        std::fs::write(&tendrils_override_json, "I'm not JSON").unwrap();

        let actual = get_tendril_overrides(&tendrils_folder.path());

        assert!(matches!(actual.unwrap_err(), GetTendrilsError::ParseError(_)));
    }

    #[test]
    #[serial]
    fn valid_json_returns_tendrils() {
        let tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "ValidJson").unwrap();
        let samples = SampleTendrils::new();
        let json = SampleTendrils::build_tendrils_json(
            &[samples.tendril_1_json].to_vec());
        let tendrils_json = &tendrils_folder.path().join("tendrils-override.json");
        std::fs::File::create(&tendrils_json).unwrap();
        std::fs::write(&tendrils_json, &json).unwrap();

        let expected = [samples.tendril_1].to_vec();

        let actual  = get_tendril_overrides(&tendrils_folder.path()).unwrap();

        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod parse_tendrils_tests {
    use super::parse_tendrils;
    use super::SampleTendrils;

    #[test]
    fn empty_string_returns_error() {
        let given = "";

        assert!(parse_tendrils(&given).is_err());
    }

    #[test]
    fn invalid_json_returns_error() {
        let given = "I'm not JSON";

        assert!(parse_tendrils(&given).is_err());
    }

    #[test]
    fn tendril_json_not_in_array_returns_error() {
        let given = SampleTendrils::new().tendril_1_json;

        assert!(parse_tendrils(&given).is_err());
    }

    #[test]
    fn json_missing_field_returns_error() {
        let original_tendril_json = SampleTendrils::new().tendril_1_json;
        let partial_tendril_json = original_tendril_json
            .replace(r#""name": "settings.json","#, "");

        let given = SampleTendrils::build_tendrils_json(
            &[partial_tendril_json.clone()].to_vec());

        let actual = parse_tendrils(&given);

        assert_ne!(&original_tendril_json, &partial_tendril_json);
        assert!(actual.is_err());
    }

    #[test]
    fn empty_json_array_returns_empty() {
        let given = "[]";
        let expected = [].to_vec();

        let actual = parse_tendrils(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn single_tendril_in_json_returns_tendril() {
        let samples = SampleTendrils::new();
        let given = SampleTendrils::build_tendrils_json(
            &[samples.tendril_1_json].to_vec());

        let expected = [samples.tendril_1].to_vec();

        let actual = parse_tendrils(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn multiple_tendrils_in_json_returns_tendrils() {
        let samples = SampleTendrils::new();
        let given = SampleTendrils::build_tendrils_json(
            &[samples.tendril_1_json, samples.tendril_2_json].to_vec());

        let expected = [samples.tendril_1, samples.tendril_2].to_vec();

        let actual = parse_tendrils(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn ignores_extra_json_field_returns_tendril() {
        let samples = SampleTendrils::new();
        let original_tendril_json = SampleTendrils::new().tendril_1_json;
        let extra_field_tendril_json = original_tendril_json
            .replace(r#""name": "settings.json","#, r#""name": "settings.json", "extra field": true,"#);

        let given = SampleTendrils::build_tendrils_json(
            &[extra_field_tendril_json.clone()].to_vec());

        let expected = [samples.tendril_1].to_vec();
        let actual = parse_tendrils(&given).unwrap();

        assert_ne!(original_tendril_json, extra_field_tendril_json);
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod resolve_overrides_tests {
    use super::resolve_overrides;
    use super::SampleTendrils;

    #[test]
    fn empty_overrides_returns_global() {
        let samples = SampleTendrils::new();
        let globals = [samples.tendril_1.clone(), samples.tendril_1.clone()].to_vec();
        let overrides = [].to_vec();

        let actual = resolve_overrides(&globals, &overrides);

        assert_eq!(actual, globals);
    }

    #[test]
    fn overrides_not_matching_globals_are_ignored() {
        let samples = SampleTendrils::new();
        let globals = [samples.tendril_1.clone()].to_vec();
        let mut misc_override = samples.tendril_1.clone();
        misc_override.app = "I don't exist".to_string();
        misc_override.name = "Me neither".to_string();
        let overrides = [misc_override].to_vec();

        let actual = resolve_overrides(&globals, &overrides);

        assert_eq!(actual, globals);
    }

    #[test]
    fn overrides_matching_globals_override_globals() {
        let samples = SampleTendrils::new();
        let globals = [
            samples.tendril_1.clone(),
            samples.tendril_2.clone()].to_vec();

        let mut override_tendril = samples.tendril_1.clone();
        override_tendril.parent_dirs_mac = ["Some/override/path".to_string()].to_vec();
        override_tendril.parent_dirs_windows = ["Some\\override\\path".to_string()].to_vec();
        let overrides = [override_tendril.clone()].to_vec();
        
        let expected = [override_tendril, samples.tendril_2.clone()].to_vec();

        let actual = resolve_overrides(&globals, &overrides);

        assert_eq!(actual, expected);
    }
}