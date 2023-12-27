use std::path::{Path, PathBuf};
mod tendril;
use tendril::Tendril;
mod errors;
use errors::{GetTendrilsError, ResolvePathError};

// TODO: Recursively look through all parent folders
// TODO: If it can't be found in the current path, check in an env variable
pub fn get_tendrils_folder(starting_path: &Path) -> Option<PathBuf> {
    if is_tendrils_folder(starting_path) {
        Some(starting_path.to_owned())
    }
    else {
        None
    }
}

pub fn get_tendrils(
    tendrils_folder: &Path,
) -> Result<Vec<Tendril>, GetTendrilsError> {
    let tendrils_file_path = Path::new(&tendrils_folder).join("tendrils.json");
    let tendrils_file_contents = std::fs::read_to_string(tendrils_file_path)?;
    let tendrils = parse_tendrils(&tendrils_file_contents)?;
    Ok(tendrils)
}

pub fn get_tendril_overrides(
    tendrils_folder: &Path,
) -> Result<Vec<Tendril>, GetTendrilsError> {
    let tendrils_file_path =
        Path::new(&tendrils_folder).join("tendrils-override.json");

    let tendrils_file_contents = if tendrils_file_path.is_file() {
        std::fs::read_to_string(tendrils_file_path)?
    }
    else {
        return Ok([].to_vec());
    };

    let tendrils = parse_tendrils(&tendrils_file_contents)?;
    Ok(tendrils)
}

pub fn is_tendrils_folder(dir: &Path) -> bool {
    dir.join("tendrils.json").is_file()
}

/// # Arguments
/// - `json` - JSON array of Tendrils
fn parse_tendrils(json: &str) -> Result<Vec<Tendril>, serde_json::Error> {
    serde_json::from_str::<Vec<Tendril>>(json)
}

/// Returns a list of all Tendrils after replacing global ones with any
/// applicable overrides.
/// # Arguments
/// - `global` - The set of Tendrils (typically defined in tendrils.json)
/// - `overrides` - The set of Tendril overrides (typically defined in
///   tendrils-overrides.json)
pub fn resolve_overrides(
    global: &[Tendril],
    overrides: &[Tendril],
) -> Vec<Tendril> {
    let mut resolved_tendrils = Vec::with_capacity(global.len());

    for tendril in global {
        let mut last_index: usize = 0;
        let overrides_iter = overrides.iter();

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

fn resolve_path_variables(path: &Path) -> Result<PathBuf, ResolvePathError> {
    let orig_string = match path.to_str() {
        Some(v) => v,
        None => return Err(ResolvePathError::PathParseError)
    };

    let username = match std::env::consts::OS {
        "macos" => std::env::var("USER")?,
        "windows" => std::env::var("USERNAME")?,
        _ => "<user>".to_string()
    };

    let resolved = orig_string.replace("<user>", &username);
    Ok(PathBuf::from(&resolved))
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
fn get_username() -> String {
    match std::env::consts::OS {
        "macos" => std::env::var("USER").unwrap(),
        "windows" => std::env::var("USERNAME").unwrap(),
        _ => unimplemented!()
    }
}

#[cfg(test)]
fn set_all_platform_paths(tendril: &mut Tendril, paths: &[PathBuf]) {
    let path_strings:Vec<String> = paths
        .iter()
        .map(|x| x.to_str().unwrap().to_string())
        .collect();

    tendril.parent_dirs_mac = path_strings.clone();
    tendril.parent_dirs_windows = path_strings;
}

#[cfg(test)]
mod is_tendrils_folder_tests {
    use super::tempdir::TempDir;
    use super::{get_disposable_folder, is_tendrils_folder};
    use serial_test::serial;

    #[test]
    #[serial] // To avoid flaky tests upon first run (before the disposable folder exists)
    fn empty_dir_returns_false() {
        let temp = TempDir::new_in(get_disposable_folder(), "Empty").unwrap();

        assert!(!is_tendrils_folder(&temp.path()));
    }

    #[test]
    #[serial]
    fn misc_other_files_only_returns_false() {
        let temp = TempDir::new_in(
            get_disposable_folder(),
            "MiscOtherFiles"
        ).unwrap();
        std::fs::File::create(temp.path().join("misc.txt")).unwrap();

        assert!(!is_tendrils_folder(&temp.path()));
    }

    #[test]
    #[serial]
    fn has_tendrils_json_dir_returns_false() {
        let temp = TempDir::new_in(
            get_disposable_folder(),
            "TendrilsJsonSubdir"
        ).unwrap();
        std::fs::create_dir(temp.path().join("tendrils.json")).unwrap();

        assert!(!is_tendrils_folder(&temp.path()));
    }

    #[test]
    #[serial]
    fn has_tendrils_json_file_returns_true() {
        let temp = TempDir::new_in(
            get_disposable_folder(),
            "EmptyTendrilsJson"
        ).unwrap();
        std::fs::File::create(temp.path().join("tendrils.json")).unwrap();

        assert!(is_tendrils_folder(&temp.path()));
    }
}

#[cfg(test)]
mod get_tendrils_folder_tests {
    use super::tempdir::TempDir;
    use super::{get_disposable_folder, get_tendrils_folder};
    use serial_test::serial;

    #[test]
    #[serial]
    fn starting_dir_not_tendrils_folder_returns_none() {
        let temp = TempDir::new_in(get_disposable_folder(), "Empty").unwrap();

        let actual = get_tendrils_folder(&temp.path());

        assert!(actual.is_none());
    }

    #[test]
    #[serial]
    fn starting_dir_is_tendrils_folder_returns_starting_dir() {
        let temp = TempDir::new_in(
            get_disposable_folder(),
            "EmptyTendrilsJson"
        ).unwrap();
        std::fs::File::create(temp.path().join("tendrils.json")).unwrap();

        let actual = get_tendrils_folder(&temp.path()).unwrap();

        assert_eq!(actual, temp.path());
    }
}

#[cfg(test)]
mod get_tendrils_tests {
    use super::tempdir::TempDir;
    use super::{
        get_disposable_folder,
        get_tendrils,
        GetTendrilsError,
        SampleTendrils,
        Tendril,
    };
    use serial_test::serial;
    use std::matches;

    #[test]
    #[serial]
    fn no_tendrils_json_file_returns_err() {
        let temp = TempDir::new_in(get_disposable_folder(), "Empty").unwrap();

        let actual = get_tendrils(&temp.path());

        assert!(matches!(actual.unwrap_err(), GetTendrilsError::IoError(_)));
    }

    #[test]
    #[serial]
    fn invalid_json_returns_err() {
        let tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "InvalidTendrilsJson"
        ) .unwrap();

        let tendrils_json = &tendrils_folder.path().join("tendrils.json");
        std::fs::File::create(&tendrils_json).unwrap();
        std::fs::write(&tendrils_json, "I'm not JSON").unwrap();

        let actual = get_tendrils(&tendrils_folder.path());

        assert!(matches!(
            actual.unwrap_err(),
            GetTendrilsError::ParseError(_)
        ));
    }

    #[test]
    #[serial]
    fn valid_json_returns_tendrils() {
        let tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "ValidJson"
        ).unwrap();
        let json = SampleTendrils::build_tendrils_json(
            &[SampleTendrils::tendril_1_json()].to_vec(),
        );
        let tendrils_json = &tendrils_folder.path().join("tendrils.json");
        std::fs::File::create(&tendrils_json).unwrap();
        std::fs::write(&tendrils_json, &json).unwrap();

        let expected = [SampleTendrils::tendril_1()].to_vec();

        let actual: Vec<Tendril> =
            get_tendrils(&tendrils_folder.path()).unwrap();

        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod get_tendril_overrides_tests {
    use super::tempdir::TempDir;
    use super::{
        get_disposable_folder,
        get_tendril_overrides,
        GetTendrilsError,
        SampleTendrils,
    };
    use serial_test::serial;
    use std::matches;

    #[test]
    #[serial]
    fn no_tendrils_json_file_returns_empty() {
        let temp = TempDir::new_in(
            get_disposable_folder(),
            "Empty"
        ).unwrap();

        let actual = get_tendril_overrides(&temp.path()).unwrap();

        assert!(actual.is_empty())
    }

    #[test]
    #[serial]
    fn invalid_json_returns_err() {
        let tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "InvalidTendrilsOverridesJson",
        ).unwrap();

        let tendrils_override_json =
            &tendrils_folder.path().join("tendrils-override.json");
        std::fs::File::create(&tendrils_override_json).unwrap();
        std::fs::write(&tendrils_override_json, "I'm not JSON").unwrap();

        let actual = get_tendril_overrides(&tendrils_folder.path());

        assert!(matches!(
            actual.unwrap_err(),
            GetTendrilsError::ParseError(_)
        ));
    }

    #[test]
    #[serial]
    fn valid_json_returns_tendrils() {
        let tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "ValidJson"
        ).unwrap();

        let json = SampleTendrils::build_tendrils_json(
            &[SampleTendrils::tendril_1_json()].to_vec(),
        );
        let tendrils_json =
            &tendrils_folder.path().join("tendrils-override.json");
        std::fs::File::create(&tendrils_json).unwrap();
        std::fs::write(&tendrils_json, &json).unwrap();

        let expected = [SampleTendrils::tendril_1()].to_vec();

        let actual = get_tendril_overrides(&tendrils_folder.path()).unwrap();

        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod parse_tendrils_tests {
    use super::{parse_tendrils, SampleTendrils};

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
        let given = SampleTendrils::tendril_1_json();

        assert!(parse_tendrils(&given).is_err());
    }

    #[test]
    fn json_missing_field_returns_error() {
        let original_tendril_json = SampleTendrils::tendril_1_json();
        let partial_tendril_json =
            original_tendril_json.replace(r#""name": "settings.json","#, "");

        let given = SampleTendrils::build_tendrils_json(
            &[partial_tendril_json.clone()].to_vec(),
        );

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
        let given = SampleTendrils::build_tendrils_json(
            &[SampleTendrils::tendril_1_json()].to_vec(),
        );

        let expected = [SampleTendrils::tendril_1()].to_vec();

        let actual = parse_tendrils(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn multiple_tendrils_in_json_returns_tendrils() {
        let given = SampleTendrils::build_tendrils_json(
            &[
                SampleTendrils::tendril_1_json(),
                SampleTendrils::tendril_2_json(),
            ].to_vec()
        );

        let expected = [
            SampleTendrils::tendril_1(),
            SampleTendrils::tendril_2()
        ].to_vec();

        let actual = parse_tendrils(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn ignores_extra_json_field_returns_tendril() {
        let original_tendril_json = SampleTendrils::tendril_1_json();
        let extra_field_tendril_json = original_tendril_json.replace(
            r#""name": "settings.json","#,
            r#""name": "settings.json", "extra field": true,"#,
        );

        let given = SampleTendrils::build_tendrils_json(
            &[extra_field_tendril_json.clone()].to_vec(),
        );

        let expected = [SampleTendrils::tendril_1()].to_vec();
        let actual = parse_tendrils(&given).unwrap();

        assert_ne!(original_tendril_json, extra_field_tendril_json);
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod resolve_overrides_tests {
    use std::path::PathBuf;

    use super::{resolve_overrides, SampleTendrils, set_all_platform_paths, Tendril};

    #[test]
    fn empty_overrides_returns_globals() {
        let globals = [
            SampleTendrils::tendril_1(),
            SampleTendrils::tendril_1()
        ].to_vec();
        let overrides = [].to_vec();

        let actual = resolve_overrides(&globals, &overrides);

        assert_eq!(actual, globals);
    }

    #[test]
    fn empty_globals_returns_empty() {
        let globals = [].to_vec();

        let mut override_tendril = SampleTendrils::tendril_1();
        set_all_platform_paths(
            &mut override_tendril,
            &[PathBuf::from("Some").join("override").join("path")]
        );
        let overrides = [override_tendril.clone()].to_vec();

        let actual = resolve_overrides(&globals, &overrides);

        assert!(actual.is_empty());
    }

    #[test]
    fn both_empty_returns_empty() {
        let globals = [].to_vec();
        let overrides = [].to_vec();

        let actual = resolve_overrides(&globals, &overrides);

        assert!(actual.is_empty());
    }

    #[test]
    fn both_equal_returns_globals() {
        let globals = [SampleTendrils::tendril_1()].to_vec();
        let overrides = &globals;

        let actual = resolve_overrides(&globals, &overrides);

        assert_eq!(actual, globals);
    }

    #[test]
    fn overrides_not_matching_globals_are_ignored() {
        let globals = [SampleTendrils::tendril_1()].to_vec();
        let mut misc_override = SampleTendrils::tendril_1();
        misc_override.app = "I don't exist".to_string();
        misc_override.name = "Me neither".to_string();
        let overrides = [misc_override].to_vec();

        let actual = resolve_overrides(&globals, &overrides);

        assert_eq!(actual, globals);
    }

    #[test]
    fn overrides_matching_globals_override_globals() {
        let globals:Vec<Tendril> = [
            SampleTendrils::tendril_1(),
            SampleTendrils::tendril_2(),
        ].to_vec();

        let mut override_tendril = globals[0].clone();
        set_all_platform_paths(
            &mut override_tendril,
            &[PathBuf::from("Some").join("override").join("path")]
        );
        override_tendril.folder_merge = !globals[0].folder_merge;
        let overrides = [override_tendril.clone()].to_vec();

        let expected = [override_tendril, SampleTendrils::tendril_2()].to_vec();

        let actual = resolve_overrides(&globals, &overrides);

        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod resolve_path_variables_tests {
    use super::{get_username, PathBuf, resolve_path_variables };

    #[test]
    #[cfg(unix)]
    // Could not get an equivalent test working on Windows.
    // Attempted using OsString::from_wide (from std::os::windows::ffi::OsStringExt)
    // with UTF-16 characters but they were successfully converted to UTF-8 for
    // some reason
    fn non_utf_8_path_returns_path_parse_error() {
        use std::os::unix::ffi::OsStringExt;
        let non_utf8_chars = vec![
            0xC3, 0x28, 0xA9, 0x29, 0xE2, 0x82, 0xAC, 0xFF, 0xFE, 0xFD, 0xFC,
            0xD8, 0x00, 0xDC, 0x00
        ];

        let non_utf8_string = std::ffi::OsString::from_vec(non_utf8_chars);

        let given = PathBuf::from(non_utf8_string);

        let actual = resolve_path_variables(&given).unwrap_err();

        assert!(matches!(actual, super::ResolvePathError::PathParseError));
    }

    #[test]
    fn empty_path_returns_empty() {
        let given = PathBuf::from("");
        let expected = given.clone();

        let actual = resolve_path_variables(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn path_without_variables_returns_given_path() {
        let given = PathBuf::from("some/generic/path");
        let expected = given.clone();

        let actual = resolve_path_variables(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn unsupported_var_returns_given_path() {
        let given = PathBuf::from("some/<unsupported>/path");
        let expected = given.clone();

        let actual = resolve_path_variables(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn wrong_capitalized_var_returns_given_path() {
        let given = PathBuf::from("storage/<USER>/my/path");
        let expected = given.clone();

        let actual = resolve_path_variables(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn user_var_replaces_with_current_username() {
        let given = PathBuf::from("storage/<user>/my/path");
        let username = get_username();

        let expected = PathBuf::from(format!("storage/{}/my/path", username));

        let actual = resolve_path_variables(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn sandwiched_var_returns_replaced_path() {
        let given = PathBuf::from("Sandwiched<user>Var");
        let username = get_username();

        let expected = PathBuf::from(format!("Sandwiched{}Var", username));

        let actual = resolve_path_variables(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn leading_var_returns_replaced_path() {
        let given = PathBuf::from("<user>LeadingVar");
        let username = get_username();

        let expected = PathBuf::from(format!("{}LeadingVar", username));

        let actual = resolve_path_variables(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn trailing_var_returns_replaced_path() {
        let given = PathBuf::from("TrailingVar<user>");
        let username = get_username();

        let expected = PathBuf::from(format!("TrailingVar{}", username));

        let actual = resolve_path_variables(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn multiple_var_instances_replaces_all() {
        let given = PathBuf::from("storage/<user>/my/<user>/path");
        let username = get_username();

        let expected = PathBuf::from(format!("storage/{}/my/{}/path", username, username));

        let actual = resolve_path_variables(&given).unwrap();

        assert_eq!(actual, expected);
    }
}
