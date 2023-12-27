mod errors;
use errors::{GetTendrilsError, PushPullError, ResolvePathError};
use std::{path::{Path, PathBuf}, fs::create_dir_all};
mod tendril;
use tendril::Tendril;

fn copy_fso(
    from: &Path,
    to: &Path,
    folder_merge: bool
) -> Result<(), std::io::Error> {
    let mut to = to;

    if from.is_dir() {
        if !folder_merge {
            std::fs::remove_dir_all(to)?;
            create_dir_all(to)?;
        }
        // TODO: Eliminate this unwrap and test how
        // root folders are handled
        to = to.parent().unwrap();
        create_dir_all(to)?;

        let mut copy_opts = fs_extra::dir::CopyOptions::new();
        copy_opts.overwrite = true;
        copy_opts.skip_exist = false;
        match fs_extra::dir::copy(from, to, &copy_opts) {
            Ok(_v) => Ok(()),
            Err(e) => match e.kind {
                // Convert fs_extra::errors to std::io::errors
                fs_extra::error::ErrorKind::Io(e) => {
                    Err(e)
                },
                fs_extra::error::ErrorKind::PermissionDenied => {
                    let e = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
                    Err(e)
                },
                _ => Err(std::io::Error::from(std::io::ErrorKind::Other))
            }
        }
    }
    else if from.is_file() {
        // TODO: Eliminate this unwrap and test how
        // root folders are handled
        create_dir_all(to.parent().unwrap())?;

        let from_str = match from.to_str() {
            Some(v) => v,
            None => return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput))
        };
        let to_str = match to.to_str() {
            Some(v) => v,
            None => return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput))
        };
        match std::fs::copy(from_str, to_str) {
            Ok(_v) => Ok(()),
            Err(e) => Err(e)
        }
    }
    else {
        return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
    }
}

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

fn is_path(x: &str) -> bool {
    x.contains("/") || x.contains("\\")
}

pub fn is_tendrils_folder(dir: &Path) -> bool {
    dir.join("tendrils.json").is_file()
}

/// # Arguments
/// - `json` - JSON array of Tendrils
fn parse_tendrils(json: &str) -> Result<Vec<Tendril>, serde_json::Error> {
    serde_json::from_str::<Vec<Tendril>>(json)
}

// TODO: Test this function
pub fn pull<'a>(
    tendrils_folder: &Path,
    tendrils: &'a [Tendril],
) -> Vec<(&'a Tendril, Result<(), PushPullError>)> {
    let mut results = Vec::with_capacity(tendrils.len());
    for tendril in tendrils {
        let result = pull_tendril(tendrils_folder, tendril);
        results.push((tendril, result));
    }

    results
}

fn pull_tendril(
    tendrils_folder: &Path,
    tendril: &Tendril,
) -> Result<(), PushPullError> {
    if tendril.app.is_empty()
        || tendril.name.is_empty()
        || is_path(&tendril.app)
        || is_path(&tendril.name) {
        return Err(PushPullError::InvalidId);
    }

    // TODO: Consider conditional compilation instead
    // of matching on every iteration
    // TODO: Extract this path determination to a separate
    // function to use with push as well
    let sources = match std::env::consts::OS {
        "macos" => &tendril.parent_dirs_mac,
        "windows" => &tendril.parent_dirs_windows,
        _ => return Err(PushPullError::Unsupported)
    };

    if sources.is_empty() {
        return Err(PushPullError::Skipped);
    }

    let source= resolve_path_variables(&PathBuf::from(&sources[0]))?
        .join(&tendril.name);
    if tendrils_folder == source 
        || tendrils_folder.ancestors().any(|p| p == source)
        || source.ancestors().any(|p| p == tendrils_folder) {
        return Err(PushPullError::Recursion);
    }

    let dest = tendrils_folder.join(&tendril.app).join(&tendril.name);

    if (source.is_dir() && dest.is_file())
        || (source.is_file() && dest.is_dir())
        || source.is_symlink()
        || dest.is_symlink() {
        return Err(PushPullError::TypeMismatch);
    }

    Ok(copy_fso(&source, &dest, tendril.folder_merge)?)
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
fn is_empty(folder: &Path) -> bool {
    if folder.exists() {
        if !folder.is_dir() {
            panic!("Expected a folder")
        }
        return folder.read_dir().unwrap().count() == 0
    }
    true
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
mod pull_tendril_tests {
    use std::fs::{
        create_dir_all,
        File,
        metadata,
        read_to_string,
        set_permissions,
        write,
    };
    use super::{
        get_disposable_folder,
        get_username,
        is_empty,
        is_tendrils_folder,
        PathBuf,
        pull_tendril,
        SampleTendrils,
        set_all_platform_paths,
    };
    use super::errors::PushPullError;
    use super::tendril::Tendril;
    use tempdir::TempDir;

    #[test]
    fn parent_path_list_is_empty_returns_skipped_error() {
        let temp_tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "ShouldBeEmpty"
        ).unwrap().into_path();

        let mut given = SampleTendrils::tendril_1();
        set_all_platform_paths(
            &mut given,
            &[].to_vec()
        );

        let actual = pull_tendril(&temp_tendrils_folder, &given).unwrap_err();

        assert!(matches!(actual, PushPullError::Skipped));
        assert!(is_empty(&temp_tendrils_folder));
    }

    #[test]
    fn parent_path_is_empty_string_attempts_copy() {
        let temp_tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "ShouldBeEmpty"
        ).unwrap().into_path();

        let mut given = SampleTendrils::tendril_1();
        set_all_platform_paths(
            &mut given,
            &[PathBuf::from("")].to_vec()
        );

        let actual = pull_tendril(&temp_tendrils_folder, &given).unwrap_err();

        match actual {
            PushPullError::IoError(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
            _ => panic!("Wrong error type")
        }
        assert!(is_empty(&temp_tendrils_folder));
    }

    #[test]
    fn tendril_app_is_empty_returns_invalid_id_error() {
        let temp_tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "ShouldBeEmpty"
        ).unwrap().into_path();

        let given = Tendril::new("", "misc.txt");

        let actual = pull_tendril(&temp_tendrils_folder, &given);

        assert!(matches!(actual, Err(PushPullError::InvalidId)));
        assert!(is_empty(&temp_tendrils_folder))
    }

    #[test]
    fn tendril_name_is_empty_returns_invalid_id_error() {
        let temp_tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "ShouldBeEmpty"
        ).unwrap().into_path();

        let given = Tendril::new("SomeApp", "");

        let actual = pull_tendril(&temp_tendrils_folder, &given);
        
        assert!(matches!(actual, Err(PushPullError::InvalidId)));
        assert!(is_empty(&temp_tendrils_folder))
    }

    #[test]
    fn tendril_app_is_a_path_returns_invalid_id_error() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");

        let given1 = Tendril::new("Some/App", "misc.txt");
        let given2 = Tendril::new("Some\\App", "misc.txt");

        let actual1 = pull_tendril(&given_tendrils_folder, &given1);
        let actual2 = pull_tendril(&given_tendrils_folder, &given2);
        
        assert!(matches!(actual1, Err(PushPullError::InvalidId)));
        assert!(matches!(actual2, Err(PushPullError::InvalidId)));
        assert!(is_empty(&temp_parent_folder))
    }

    #[test]
    fn tendril_name_is_a_path_returns_invalid_id_error() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");

        let given1 = Tendril::new("SomeApp", "Nested/misc.txt");
        let given2 = Tendril::new("SomeApp", "Nested\\misc.txt");

        let actual1 = pull_tendril(&given_tendrils_folder, &given1);
        let actual2 = pull_tendril(&given_tendrils_folder, &given2);
        
        assert!(matches!(actual1, Err(PushPullError::InvalidId)));
        assert!(matches!(actual2, Err(PushPullError::InvalidId)));
        assert!(is_empty(&temp_parent_folder))
    }

    #[test]
    fn tendril_name_has_leading_dot_is_copied_normally() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = temp_parent_folder.join("TendrilsFolder");
        let source_file = temp_parent_folder.join(".dotfile");
        let source_folder = temp_parent_folder.join(".dotfolder");
        let dest_file = given_tendrils_folder.join("SomeApp").join(".dotfile");
        let dest_folder = given_tendrils_folder.join("SomeApp").join(".dotfolder");
        write(source_file, "Source file contents").unwrap();
        create_dir_all(&source_folder).unwrap();

        let mut given_file_tendril = Tendril::new("SomeApp", ".dotfile");
        let mut given_folder_tendril = Tendril::new("SomeApp", ".dotfolder");
        set_all_platform_paths(&mut given_file_tendril, &[temp_parent_folder.clone()]);
        set_all_platform_paths(&mut given_folder_tendril, &[temp_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given_file_tendril).unwrap();
        pull_tendril(&given_tendrils_folder, &given_folder_tendril).unwrap();

        let dest_file_contents = read_to_string(dest_file).unwrap();
        assert_eq!(dest_file_contents, "Source file contents");
        assert!(dest_folder.is_dir());
        assert!(is_empty(&dest_folder));
    }

    #[test]
    fn tendril_name_has_sandwiched_dot_is_copied_normally() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = temp_parent_folder.join("TendrilsFolder");
        let source_file = temp_parent_folder.join("sandwiched.dot.file");
        let source_folder = temp_parent_folder.join("sandwiched.dot.folder");
        let dest_file = given_tendrils_folder.join("SomeApp").join("sandwiched.dot.file");
        let dest_folder = given_tendrils_folder.join("SomeApp").join("sandwiched.dot.folder");
        write(source_file, "Source file contents").unwrap();
        create_dir_all(&source_folder).unwrap();

        let mut given_file_tendril = Tendril::new("SomeApp", "sandwiched.dot.file");
        let mut given_folder_tendril = Tendril::new("SomeApp", "sandwiched.dot.folder");
        set_all_platform_paths(&mut given_file_tendril, &[temp_parent_folder.clone()]);
        set_all_platform_paths(&mut given_folder_tendril, &[temp_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given_file_tendril).unwrap();
        pull_tendril(&given_tendrils_folder, &given_folder_tendril).unwrap();

        let dest_file_contents = read_to_string(dest_file).unwrap();
        assert_eq!(dest_file_contents, "Source file contents");
        assert!(dest_folder.is_dir());
        assert!(is_empty(&dest_folder));
    }

    #[test]
    fn tendril_name_does_not_have_dot_is_copied_normally() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = temp_parent_folder.join("TendrilsFolder");
        let source_file = temp_parent_folder.join("no_dot_file");
        let source_folder = temp_parent_folder.join("no_dot_folder");
        let dest_file = given_tendrils_folder.join("SomeApp").join("no_dot_file");
        let dest_folder = given_tendrils_folder.join("SomeApp").join("no_dot_folder");
        write(source_file, "Source file contents").unwrap();
        create_dir_all(&source_folder).unwrap();

        let mut given_file_tendril = Tendril::new("SomeApp", "no_dot_file");
        let mut given_folder_tendril = Tendril::new("SomeApp", "no_dot_folder");
        set_all_platform_paths(&mut given_file_tendril, &[temp_parent_folder.clone()]);
        set_all_platform_paths(&mut given_folder_tendril, &[temp_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given_file_tendril).unwrap();
        pull_tendril(&given_tendrils_folder, &given_folder_tendril).unwrap();

        let dest_file_contents = read_to_string(dest_file).unwrap();
        assert_eq!(dest_file_contents, "Source file contents");
        assert!(dest_folder.is_dir());
        assert!(is_empty(&dest_folder));
    }

    #[test]
    fn tendril_name_has_trailing_dot_is_copied_normally() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = temp_parent_folder.join("TendrilsFolder");
        let source_file = temp_parent_folder.join("file.");
        let source_folder = temp_parent_folder.join("folder.");
        let dest_file = given_tendrils_folder.join("SomeApp").join("file.");
        let dest_folder = given_tendrils_folder.join("SomeApp").join("folder.");
        write(source_file, "Source file contents").unwrap();
        create_dir_all(&source_folder).unwrap();

        let mut given_file_tendril = Tendril::new("SomeApp", "file.");
        let mut given_folder_tendril = Tendril::new("SomeApp", "folder.");
        set_all_platform_paths(&mut given_file_tendril, &[temp_parent_folder.clone()]);
        set_all_platform_paths(&mut given_folder_tendril, &[temp_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given_file_tendril).unwrap();
        pull_tendril(&given_tendrils_folder, &given_folder_tendril).unwrap();

        let dest_file_contents = read_to_string(dest_file).unwrap();
        assert_eq!(dest_file_contents, "Source file contents");
        assert!(dest_folder.is_dir());
        assert!(is_empty(&dest_folder));
    }

    #[test]
    fn var_in_tendrils_folder_uses_raw_path() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = temp_parent_folder.join("TendrilsFolderWithVar_<user>");
        File::create(&temp_parent_folder.join("misc.txt")).unwrap();

        let mut given = Tendril::new("SomeApp", "misc.txt");
        set_all_platform_paths(&mut given, &[temp_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given).unwrap();

        assert!(given_tendrils_folder.join("SomeApp").join("misc.txt").exists());
        assert!(given_tendrils_folder.join("SomeApp").read_dir().unwrap().count() == 1);
    }

    #[test]
    fn var_in_tendril_app_uses_raw_path() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
        let file_tendril_source = &temp_parent_folder.join("misc.txt");
        let folder_tendril_source = &temp_parent_folder.join("SourceFolder");
        let file_tendril_dest = &given_tendrils_folder.join("<user>").join("misc.txt");
        let folder_tendril_dest = &given_tendrils_folder.join("<user>").join("SourceFolder");
        write(&file_tendril_source, "Source file contents").unwrap();
        create_dir_all(folder_tendril_source).unwrap();

        let mut given_file_tendril = Tendril::new("<user>", "misc.txt");
        let mut given_folder_tendril = Tendril::new("<user>", "SourceFolder");
        set_all_platform_paths(&mut given_file_tendril, &[temp_parent_folder.clone()]);
        set_all_platform_paths(&mut given_folder_tendril, &[temp_parent_folder]);

        pull_tendril(given_tendrils_folder, &given_file_tendril).unwrap();
        pull_tendril(given_tendrils_folder, &given_folder_tendril).unwrap();

        let dest_file_contents = read_to_string(&file_tendril_dest).unwrap();
        assert_eq!(dest_file_contents, "Source file contents");
        assert!(folder_tendril_dest.exists());
    }

    #[test]
    fn var_in_tendril_name_uses_raw_path() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
        let file_tendril_source = &temp_parent_folder.join("<user>.txt");
        let folder_tendril_source = &temp_parent_folder.join("<user>");
        let file_tendril_dest = &given_tendrils_folder.join("SomeApp").join("<user>.txt");
        let folder_tendril_dest = &given_tendrils_folder.join("SomeApp").join("<user>");
        write(&file_tendril_source, "Source file contents").unwrap();
        create_dir_all(folder_tendril_source).unwrap();

        let mut given_file_tendril = Tendril::new("SomeApp", "<user>.txt");
        let mut given_folder_tendril = Tendril::new("SomeApp", "<user>");
        set_all_platform_paths(&mut given_file_tendril, &[temp_parent_folder.clone()]);
        set_all_platform_paths(&mut given_folder_tendril, &[temp_parent_folder]);

        pull_tendril(given_tendrils_folder, &given_file_tendril).unwrap();
        pull_tendril(given_tendrils_folder, &given_folder_tendril).unwrap();

        let dest_file_contents = read_to_string(&file_tendril_dest).unwrap();
        assert_eq!(dest_file_contents, "Source file contents");
        assert!(folder_tendril_dest.exists());
    }

    #[test]
    fn unsupported_var_in_parent_path_uses_raw_path() {
        let temp_grandparent_folder = TempDir::new_in(
            get_disposable_folder(),
            "GrandparentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = temp_grandparent_folder.join("TendrilsFolder");
        let source = temp_grandparent_folder.join("<unsupported>").join("misc.txt");
        let dest = given_tendrils_folder.join("SomeApp").join("misc.txt");
        create_dir_all(source.parent().unwrap()).unwrap();
        write(source, "Source file contents").unwrap();

        let mut given = Tendril::new("SomeApp", "misc.txt");
        set_all_platform_paths(
            &mut given,
            &[temp_grandparent_folder.join("<unsupported>")]
        );

        pull_tendril(&given_tendrils_folder, &given).unwrap();

        let dest_file_contents = read_to_string(dest).unwrap();
        assert_eq!(dest_file_contents, "Source file contents");
    }

    #[test]
    fn supported_var_in_parent_path_is_resolved() {
        let temp_grandparent_folder = TempDir::new_in(
            get_disposable_folder(),
            "GrandparentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_grandparent_folder.join("TendrilsFolder");
        let source = &temp_grandparent_folder.join(get_username()).join("misc.txt");
        let dest = &given_tendrils_folder.join("SomeApp").join("misc.txt");
        create_dir_all(&source.parent().unwrap()).unwrap();
        write(&source, "Source file contents").unwrap();

        let mut given = Tendril::new("SomeApp", "misc.txt");
        set_all_platform_paths(&mut given, &[temp_grandparent_folder.join("<user>")]);

        pull_tendril(&given_tendrils_folder, &given).unwrap();

        let dest_file_contents = read_to_string(&dest).unwrap();
        assert_eq!(dest_file_contents, "Source file contents");
    }

    #[test]
    fn resolved_source_path_doesnt_exist_returns_io_error_not_found() {
        let temp_tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "ShouldBeEmpty"
        ).unwrap().into_path();

        let mut given = Tendril::new("SomeApp", "misc.txt");
        set_all_platform_paths(
            &mut given,
            &[PathBuf::from("SomePathThatDoesNotExist")].to_vec()
        );

        let actual = pull_tendril(&temp_tendrils_folder, &given).unwrap_err();
        match actual {
            PushPullError::IoError(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
            _ => panic!("Wrong error kind"),
        }

        assert!(is_empty(&temp_tendrils_folder));
    }

    #[test]
    fn resolved_source_path_is_given_tendrils_folder_returns_recursion_error() {
        let temp_grandparent_folder = TempDir::new_in(
            get_disposable_folder(),
            "GrandparentFolder"
        ).unwrap().into_path();
        let given_parent_folder = temp_grandparent_folder.join("<user>");
        let given_tendrils_folder = temp_grandparent_folder
            .join(get_username())
            .join("TendrilsFolder");
        create_dir_all(&temp_grandparent_folder.join(get_username())).unwrap();

        let mut given = Tendril::new("SomeApp", "TendrilsFolder");
        set_all_platform_paths(&mut given, &[given_parent_folder]);

        let actual = pull_tendril(&given_tendrils_folder, &given);

        assert!(matches!(actual, Err(PushPullError::Recursion)));
        assert!(is_empty(&given_tendrils_folder));
    }

    #[test]
    fn resolved_source_path_is_ancestor_to_given_tendrils_folder_returns_recursion_error() {
        let temp_grandparent_folder = TempDir::new_in(
            get_disposable_folder(),
            "GrandparentFolder"
        ).unwrap().into_path();
        let given_parent_folder = temp_grandparent_folder.join("<user>");
        let given_tendrils_folder = temp_grandparent_folder
            .join(get_username())
            .join("Nested1")
            .join("Nested2")
            .join("Nested3")
            .join("TendrilsFolder");
        create_dir_all(&temp_grandparent_folder.join(get_username())).unwrap();

        let mut given = Tendril::new("SomeApp", "Nested1");
        set_all_platform_paths(&mut given, &[given_parent_folder]);

        let actual = pull_tendril(&given_tendrils_folder, &given);

        assert!(matches!(actual, Err(PushPullError::Recursion)));
        assert!(is_empty(&given_tendrils_folder));
    }

    #[test]
    fn resolved_source_path_is_sibling_to_given_tendrils_folder_copies_normally() {
        let temp_grandparent_folder = TempDir::new_in(
            get_disposable_folder(),
            "GrandparentFolder"
        ).unwrap().into_path();
        let given_parent_folder = temp_grandparent_folder.join("<user>");
        let given_tendrils_folder = temp_grandparent_folder
            .join(get_username())
            .join("TendrilsFolder");
        create_dir_all(&temp_grandparent_folder
            .join(get_username())
            .join("SiblingFolder")
        ).unwrap();

        let mut given = Tendril::new("SomeApp", "SiblingFolder");
        set_all_platform_paths(&mut given, &[given_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given).unwrap();

        assert!(given_tendrils_folder
            .join("SomeApp")
            .join("SiblingFolder")
            .exists()
        );
    }

    #[test]
    fn resolved_source_path_is_direct_child_of_given_tendrils_folder_returns_recursion_error() {
        let temp_grandparent_folder = TempDir::new_in(
            get_disposable_folder(),
            "GrandparentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = temp_grandparent_folder
            .join(get_username())
            .join("TendrilsFolder");
        let given_parent_folder = temp_grandparent_folder
            .join("<user>")
            .join("TendrilsFolder");
        let source = given_tendrils_folder.join("misc.txt");
        create_dir_all(&given_tendrils_folder).unwrap();
        write(&source, "Source file contents").unwrap();

        let mut given = Tendril::new("SomeApp", "misc.txt");
        set_all_platform_paths(&mut given, &[given_parent_folder]);

        let actual = pull_tendril(&given_tendrils_folder, &given);

        assert!(matches!(actual, Err(PushPullError::Recursion)));
        assert_eq!(read_to_string(source).unwrap(), "Source file contents");
        assert!(given_tendrils_folder.read_dir().unwrap().into_iter().count() == 1);
    }

    #[test]
    fn resolved_source_path_is_nested_child_of_given_tendrils_folder_returns_recursion_error() {
        let temp_grandparent_folder = TempDir::new_in(
            get_disposable_folder(),
            "GrandparentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = temp_grandparent_folder
            .join(get_username())
            .join("TendrilsFolder");
        let given_parent_folder = temp_grandparent_folder
            .join("<user>")
            .join("TendrilsFolder")
            .join("Nested1")
            .join("Nested2")
            .join("Nested3");
        let source = given_tendrils_folder
            .join("Nested1")
            .join("Nested2")
            .join("Nested3")
            .join("misc.txt");
        create_dir_all(&source.parent().unwrap()).unwrap();
        write(&source, "Source file contents").unwrap();

        let mut given = Tendril::new("SomeApp", "misc.txt");
        set_all_platform_paths(&mut given, &[given_parent_folder]);

        let actual = pull_tendril(&given_tendrils_folder, &given);

        assert!(matches!(actual, Err(PushPullError::Recursion)));
        assert_eq!(read_to_string(source).unwrap(), "Source file contents");
        assert!(given_tendrils_folder.read_dir().unwrap().into_iter().count() == 1);
    }

    #[test]
    fn resolved_source_path_is_another_tendrils_folder_still_copies() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
        let source = &temp_parent_folder.join("AnotherTendrilsFolder");
        create_dir_all(&source).unwrap();
        write(&source.join("tendrils.json"), "").unwrap();
        assert!(is_tendrils_folder(&source));

        let mut given = Tendril::new("SomeApp", "AnotherTendrilsFolder");
        set_all_platform_paths(&mut given, &[temp_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given).unwrap();

        assert!(given_tendrils_folder.join("SomeApp")
            .join("AnotherTendrilsFolder").join("tendrils.json").exists()
        );
        assert!(given_tendrils_folder.join("SomeApp")
            .read_dir().unwrap().count() == 1
        );
    }

    #[test]
    fn resolved_source_path_is_file_and_dest_is_dir_returns_type_mismatch_error() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
        let source = &temp_parent_folder.join("misc");
        let dest = given_tendrils_folder.join("SomeApp").join("misc");
        write(&source, "Source file contents").unwrap();
        create_dir_all(&dest).unwrap();

        let mut given = Tendril::new("SomeApp", "misc");
        set_all_platform_paths(&mut given, &[temp_parent_folder]);

        let actual = pull_tendril(&given_tendrils_folder, &given).unwrap_err();

        let source_file_contents = read_to_string(source).unwrap();
        assert_eq!(source_file_contents, "Source file contents");
        assert!(matches!(actual, PushPullError::TypeMismatch));
        assert!(dest.is_dir());
        assert!(is_empty(&dest));
    }

    #[test]
    fn resolved_source_path_is_dir_and_dest_is_file_returns_type_mismatch_error() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
        let source = &temp_parent_folder.join("misc");
        let dest = given_tendrils_folder.join("SomeApp").join("misc");
        create_dir_all(&source).unwrap();
        create_dir_all(&dest.parent().unwrap()).unwrap();
        write(&dest, "Dest file contents").unwrap();

        let mut given = Tendril::new("SomeApp", "misc");
        set_all_platform_paths(&mut given, &[temp_parent_folder]);

        let actual = pull_tendril(&given_tendrils_folder, &given).unwrap_err();

        let dest_file_content = read_to_string(dest).unwrap();
        assert_eq!(dest_file_content, "Dest file contents");
        assert!(matches!(actual, PushPullError::TypeMismatch));
        assert!(source.is_dir());
        assert!(is_empty(&source));
    }

    #[test]
    fn resolved_source_path_is_symlink_returns_type_mismatch_error() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = temp_parent_folder.join("TendrilsFolder");
        let original_file = temp_parent_folder.join("original.txt");
        let original_folder = temp_parent_folder.join("original");
        let symlink_file = temp_parent_folder.join("symfile");
        let symlink_folder = temp_parent_folder.join("symdir");
        write(&original_file, "Original file contents").unwrap();
        create_dir_all(&original_folder).unwrap();

        // Create symlinks
        #[cfg(unix)]
        use std::os::unix::fs::symlink;
        #[cfg(unix)]
        symlink(original_file, symlink_file).unwrap();
        #[cfg(unix)]
        symlink(original_folder, symlink_folder).unwrap();

        #[cfg(windows)]
        use std::os::windows::fs::{symlink_dir, symlink_file};
        #[cfg(windows)]
        unimplemented!();

        // Note, each of these tendrils could be considered a file tendril or a folder tendril
        // as the type is determined by the type of the source file system object
        let mut given_pointing_to_symfile = Tendril::new("SomeApp", "symfile");
        let mut given_pointing_to_symdir = Tendril::new("SomeApp", "symdir");
        set_all_platform_paths(&mut given_pointing_to_symfile, &[temp_parent_folder.clone()]);
        set_all_platform_paths(&mut given_pointing_to_symdir, &[temp_parent_folder]);

        let actual_1 = pull_tendril(
            &given_tendrils_folder,
            &given_pointing_to_symfile
        ).unwrap_err();
        let actual_2 = pull_tendril(
            &given_tendrils_folder, &given_pointing_to_symdir
        ).unwrap_err();

        assert!(matches!(actual_1, PushPullError::TypeMismatch));
        assert!(matches!(actual_2, PushPullError::TypeMismatch));
        assert!(is_empty(&given_tendrils_folder));
    }

    #[test]
    fn dest_is_symlink_returns_type_mismatch_error() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = temp_parent_folder.join("TendrilsFolder");
        let source_file = temp_parent_folder.join("symfile");
        let source_folder = temp_parent_folder.join("symdir");
        let original_file = given_tendrils_folder.join("SomeApp").join("original.txt");
        let original_folder = given_tendrils_folder
            .join("SomeApp")
            .join("original");
        let symlink_file = given_tendrils_folder
            .join("SomeApp")
            .join("symfile");
        let symlink_folder = given_tendrils_folder
            .join("SomeApp")
            .join("symdir");
        create_dir_all(&original_folder).unwrap();
        create_dir_all(&source_folder).unwrap();
        write(&original_file, "Original file contents").unwrap();
        write(&source_file, "Source file contents").unwrap();

        // Create symlinks
        #[cfg(unix)]
        use std::os::unix::fs::symlink;
        #[cfg(unix)]
        symlink(original_file, symlink_file).unwrap();
        #[cfg(unix)]
        symlink(original_folder, symlink_folder).unwrap();

        #[cfg(windows)]
        use std::os::windows::fs::{symlink_dir, symlink_file};
        #[cfg(windows)]
        unimplemented!();

        // Note, each of these tendrils could be considered a file tendril or a folder tendril
        // as the type is determined by the type of the source file system object
        let mut given_pointing_to_symfile = Tendril::new("SomeApp", "symfile");
        let mut given_pointing_to_symdir = Tendril::new("SomeApp", "symdir");
        set_all_platform_paths(&mut given_pointing_to_symfile, &[temp_parent_folder.clone()]);
        set_all_platform_paths(&mut given_pointing_to_symdir, &[temp_parent_folder]);

        let actual_1 = pull_tendril(
            &given_tendrils_folder,
            &given_pointing_to_symfile
        ).unwrap_err();
        let actual_2 = pull_tendril(
            &given_tendrils_folder, &given_pointing_to_symdir
        ).unwrap_err();

        assert!(matches!(actual_1, PushPullError::TypeMismatch));
        assert!(matches!(actual_2, PushPullError::TypeMismatch));
    }

    #[test]
    fn no_read_access_from_source_file_returns_io_error_permission_denied() {
        let temp_tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "TendrilsFolder"
        ).unwrap().into_path();
        // Note: This test sample is not version controlled and must first
        // be created using the setup script - See dev/setup-tendrils.nu
        let source = &temp_tendrils_folder
            .parent().unwrap()
            .parent().unwrap()
            .parent().unwrap()
            .join("tests")
            .join("samples")
            .join("NoReadAccess")
            .join("no_read_access.txt");

        let mut given = Tendril::new("SomeApp", "no_read_access.txt");
        set_all_platform_paths(&mut given, &[source.parent().unwrap().to_owned()]);

        let actual = pull_tendril(&temp_tendrils_folder, &given);

        match actual {
            Err(PushPullError::IoError(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied)
            },
            _ => panic!()
        }
        assert!(is_empty(&temp_tendrils_folder.join("SomeApp")));
    }

    #[test]
    fn no_read_access_from_source_folder_returns_io_error_permission_denied() {
        let temp_tendrils_folder = TempDir::new_in(
            get_disposable_folder(),
            "TendrilsFolder"
        ).unwrap().into_path();
        // Note: This test sample is not version controlled and must first
        // be created using the setup script - See dev/setup-tendrils.nu
        let source = &temp_tendrils_folder
            .parent().unwrap()
            .parent().unwrap()
            .parent().unwrap()
            .join("tests")
            .join("samples")
            .join("NoReadAccess")
            .join("no_read_access_folder");

        let mut given = Tendril::new("SomeApp", "no_read_access_folder");
        set_all_platform_paths(&mut given, &[source.parent().unwrap().to_owned()]);

        let actual = pull_tendril(&temp_tendrils_folder, &given);

        match actual {
            Err(PushPullError::IoError(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied)
            },
            _ => panic!()
        }
        assert!(is_empty(&temp_tendrils_folder.join("SomeApp")));
    }

    #[test]
    fn no_write_access_at_dest_file_returns_io_error_permission_denied() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
        let source = &temp_parent_folder.join("misc.txt");
        let dest = &given_tendrils_folder.join("SomeApp").join("misc.txt");
        File::create(&source).unwrap();
        create_dir_all(&dest.parent().unwrap()).unwrap();
        write(&dest, "Don't touch me").unwrap();

        // Set file read-only
        let mut perms = metadata(&dest).unwrap().permissions();
        perms.set_readonly(true);
        set_permissions(&dest, perms).unwrap();

        let mut given = Tendril::new("SomeApp", "misc.txt");
        set_all_platform_paths(&mut given, &[temp_parent_folder]);

        let actual = pull_tendril(&given_tendrils_folder, &given);

        match actual {
            Err(PushPullError::IoError(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied)
            },
            _ => panic!()
        }
        let dest_file_contents = read_to_string(&dest).unwrap();
        assert_eq!(dest_file_contents, "Don't touch me");
    }

    #[test]
    fn folder_merge_false_w_file_tendril_overwrites_dest_file() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = temp_parent_folder.join("TendrilsFolder");
        let source = &temp_parent_folder.join("misc.txt");
        let dest = &given_tendrils_folder.join("SomeApp").join("misc.txt");
        File::create(&source).unwrap();
        write(&source, "Source file contents").unwrap();
        create_dir_all(&dest.parent().unwrap()).unwrap();
        File::create(&dest).unwrap();

        let mut given = Tendril::new("SomeApp", "misc.txt");
        given.folder_merge = false;
        set_all_platform_paths(&mut given, &[temp_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given).unwrap();

        let dest_file_contents = read_to_string(&dest).unwrap();
        assert_eq!(dest_file_contents, "Source file contents");
    }

    #[test]
    fn folder_merge_true_w_file_tendril_overwrites_dest_file() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = temp_parent_folder.join("TendrilsFolder");
        let source = &temp_parent_folder.join("misc.txt");
        let dest = &given_tendrils_folder.join("SomeApp").join("misc.txt");
        File::create(&source).unwrap();
        write(&source, "Source file contents").unwrap();
        create_dir_all(&dest.parent().unwrap()).unwrap();
        File::create(&dest).unwrap();

        let mut given = Tendril::new("SomeApp", "misc.txt");
        given.folder_merge = true;
        set_all_platform_paths(&mut given, &[temp_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given).unwrap();

        let dest_file_contents = read_to_string(&dest).unwrap();
        assert_eq!(dest_file_contents, "Source file contents");
    }

    #[test]
    fn folder_merge_false_w_folder_tendril_overwrites_dest_folder_recursively() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
        let source= &temp_parent_folder.join("SourceFolder");
        let nested_folder= &source.join("NestedFolder");
        let source_misc_file = source.join("misc.txt");
        let source_nested_file = source.join("NestedFolder").join("nested.txt");
        let source_new_nested_file = source.join("NestedFolder").join("new_nested.txt");
        let dest_misc_file = given_tendrils_folder
            .join("SomeApp")
            .join("SourceFolder")
            .join("misc.txt");
        let dest_nested_file = given_tendrils_folder
            .join("SomeApp")
            .join("SourceFolder")
            .join("NestedFolder")
            .join("nested.txt");
        let dest_new_nested_file = given_tendrils_folder
            .join("SomeApp")
            .join("SourceFolder")
            .join("NestedFolder")
            .join("new_nested.txt");
        let dest_extra_nested_file = given_tendrils_folder
            .join("SomeApp")
            .join("SourceFolder")
            .join("NestedFolder")
            .join("extra_nested.txt"); // Should no longer exist
        create_dir_all(&nested_folder).unwrap();
        create_dir_all(dest_nested_file.parent().unwrap()).unwrap();
        write(&source_misc_file, "Source misc file").unwrap();
        write(&source_nested_file, "Source nested file").unwrap();
        write(&source_new_nested_file, "I'm not in the tendrils folder").unwrap();
        write(&dest_misc_file, "Existing misc file").unwrap();
        write(&dest_nested_file, "Existing nested file").unwrap();
        write(&dest_extra_nested_file, "I'm not in the source folder").unwrap();

        let mut given = Tendril::new("SomeApp", "SourceFolder");
        given.folder_merge = false;
        set_all_platform_paths(&mut given, &[temp_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given).unwrap();

        let dest_misc_contents = read_to_string(dest_misc_file).unwrap();
        let dest_nested_contents = read_to_string(dest_nested_file).unwrap();
        let dest_new_nested_contents = read_to_string(dest_new_nested_file).unwrap();
        assert_eq!(dest_misc_contents, "Source misc file");
        assert_eq!(dest_nested_contents, "Source nested file");
        assert_eq!(dest_new_nested_contents, "I'm not in the tendrils folder");
        assert!(!dest_extra_nested_file.exists());
    }

    #[test]
    fn folder_merge_true_w_folder_tendril_merges_w_dest_folder_recursively() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
        let source= &temp_parent_folder.join("SourceFolder");
        let nested_folder= &source.join("NestedFolder");
        let source_misc_file = source.join("misc.txt");
        let source_nested_file = source.join("NestedFolder").join("nested.txt");
        let source_new_nested_file = source.join("NestedFolder").join("new_nested.txt");
        let dest_misc_file = given_tendrils_folder
            .join("SomeApp")
            .join("SourceFolder")
            .join("misc.txt");
        let dest_nested_file = given_tendrils_folder
            .join("SomeApp")
            .join("SourceFolder")
            .join("NestedFolder")
            .join("nested.txt");
        let dest_new_nested_file = given_tendrils_folder
            .join("SomeApp")
            .join("SourceFolder")
            .join("NestedFolder")
            .join("new_nested.txt");
        let dest_extra_nested_file = given_tendrils_folder
            .join("SomeApp")
            .join("SourceFolder")
            .join("NestedFolder")
            .join("extra_nested.txt");
        create_dir_all(&nested_folder).unwrap();
        create_dir_all(dest_nested_file.parent().unwrap()).unwrap();
        write(&source_misc_file, "Source misc file").unwrap();
        write(&source_nested_file, "Source nested file").unwrap();
        write(&source_new_nested_file, "I'm not in the tendrils folder").unwrap();
        write(&dest_misc_file, "Existing misc file").unwrap();
        write(&dest_nested_file, "Existing nested file").unwrap();
        write(&dest_extra_nested_file, "I'm not in the source folder").unwrap();

        let mut given = Tendril::new("SomeApp", "SourceFolder");
        given.folder_merge = true;
        set_all_platform_paths(&mut given, &[temp_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given).unwrap();

        let dest_misc_contents = read_to_string(dest_misc_file).unwrap();
        let dest_nested_contents = read_to_string(dest_nested_file).unwrap();
        let dest_new_nested_contents = read_to_string(dest_new_nested_file).unwrap();
        let dest_extra_nested_contents = read_to_string(dest_extra_nested_file).unwrap();
        assert_eq!(dest_misc_contents, "Source misc file");
        assert_eq!(dest_nested_contents, "Source nested file");
        assert_eq!(dest_new_nested_contents, "I'm not in the tendrils folder");
        assert_eq!(dest_extra_nested_contents, "I'm not in the source folder");
    }

    #[test]
    fn tendrils_folder_doesnt_exist_creates_folder_and_subfolders_first() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder
            .join("TendrilsFolderThatDoesNotExistYet");
        let source = &temp_parent_folder.join("misc.txt");
        let dest = &given_tendrils_folder
            .join("SomeApp")
            .join("misc.txt");
        write(&source, "Source file contents").unwrap();

        let mut given = Tendril::new("SomeApp", "misc.txt");
        set_all_platform_paths(&mut given, &[temp_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given).unwrap();

        let dest_file_contents = read_to_string(&dest).unwrap();
        assert_eq!(dest_file_contents, "Source file contents");
    }

    #[test]
    fn file_tendril_source_is_unchanged() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
        let source= &temp_parent_folder.join("misc.txt");
        write(source, "Source file contents").unwrap();
        let dest = given_tendrils_folder
            .join("SomeApp")
            .join("misc.txt");

        let mut given = Tendril::new("SomeApp", "misc.txt");
        set_all_platform_paths(&mut given, &[temp_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given).unwrap();

        let dest_contents = read_to_string(dest).unwrap();
        assert_eq!(dest_contents, "Source file contents");
        
        // Check that source is unchanged
        let source_contents = read_to_string(source).unwrap();
        assert_eq!(source_contents, "Source file contents");
    }

    #[test]
    fn other_tendrils_in_same_app_folder_are_unchanged() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
        let source= &temp_parent_folder.join("misc.txt");
        let some_other_tendril= &given_tendrils_folder.join("SomeApp").join("other.txt");
        create_dir_all(given_tendrils_folder.join("SomeApp")).unwrap();
        write(source, "Source file contents").unwrap();
        write(some_other_tendril, "Another tendril from the same app").unwrap();

        let mut given = Tendril::new("SomeApp", "misc.txt");
        set_all_platform_paths(&mut given, &[temp_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given).unwrap();
        
        // Check that other tendril is unchanged
        let some_other_tendril_contents = read_to_string(some_other_tendril).unwrap();
        assert_eq!(some_other_tendril_contents, "Another tendril from the same app");
    }

    #[test]
    fn folder_tendril_copies_all_contents_recursively_and_source_is_unchanged() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
        let source= &temp_parent_folder.join("SourceFolder");
        let nested_folder= &source.join("NestedFolder");
        create_dir_all(&nested_folder).unwrap();
        write(&source.join("misc.txt"), "Misc file contents").unwrap();
        write(&nested_folder.join("nested.txt"), "Nested file contents").unwrap();
        let dest_misc_file = given_tendrils_folder
            .join("SomeApp")
            .join("SourceFolder")
            .join("misc.txt");
        let dest_nested_file = given_tendrils_folder
            .join("SomeApp")
            .join("SourceFolder")
            .join("NestedFolder")
            .join("nested.txt");

        let mut given = Tendril::new("SomeApp", "SourceFolder");
        set_all_platform_paths(&mut given, &[temp_parent_folder]);

        pull_tendril(&given_tendrils_folder, &given).unwrap();

        let dest_misc_contents = read_to_string(dest_misc_file).unwrap();
        let dest_nested_contents = read_to_string(dest_nested_file).unwrap();
        assert_eq!(dest_misc_contents, "Misc file contents");
        assert_eq!(dest_nested_contents, "Nested file contents");
        
        // Check that source is unchanged
        let source_misc_contents = read_to_string(source.join("misc.txt")).unwrap();
        let source_nested_contents = read_to_string(nested_folder.join("nested.txt")).unwrap();
        assert_eq!(source_misc_contents, "Misc file contents");
        assert_eq!(source_nested_contents, "Nested file contents");
    }

    #[test]
    fn copies_from_correct_platform_paths() {
        let temp_grandparent_folder = TempDir::new_in(
            get_disposable_folder(),
            "GrandparentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_grandparent_folder.join("TendrilsFolder");
        let parent_mac = temp_grandparent_folder.join("Mac");
        let parent_win = temp_grandparent_folder.join("Windows");
        let source_mac= parent_mac.join("misc.txt");
        let source_win= parent_win.join("misc.txt");
        let dest = given_tendrils_folder.join("SomeApp").join("misc.txt");
        create_dir_all(&parent_mac).unwrap();
        create_dir_all(&parent_win).unwrap();
        write(source_mac, "Mac file contents").unwrap();
        write(source_win, "Windows file contents").unwrap();

        let mut given = Tendril::new("SomeApp", "misc.txt");
        given.parent_dirs_mac = [parent_mac.to_str().unwrap().to_string()].to_vec();
        given.parent_dirs_windows = [parent_win.to_str().unwrap().to_string()].to_vec();

        pull_tendril(given_tendrils_folder, &given).unwrap();

        let dest_contents = read_to_string(dest).unwrap();
        match std::env::consts::OS {
            "macos" => assert_eq!(dest_contents, "Mac file contents"),
            "windows" => assert_eq!(dest_contents, "Windows file contents"),
            _ => unimplemented!()
        }
    }

    #[test]
    fn multiple_paths_only_copies_first() {
        let temp_grandparent_folder = TempDir::new_in(
            get_disposable_folder(),
            "GrandparentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = temp_grandparent_folder.join("TendrilsFolder");
        let given_parent_folder_1 = temp_grandparent_folder.join("Parent1");
        let given_parent_folder_2 = temp_grandparent_folder.join("Parent2");
        create_dir_all(&given_tendrils_folder).unwrap();
        create_dir_all(&given_parent_folder_1).unwrap();
        create_dir_all(&given_parent_folder_2).unwrap();
        write(given_parent_folder_1.join("misc.txt"), "Copy me!").unwrap();
        write(given_parent_folder_2.join("misc.txt"), "Don't copy me!").unwrap();

        let mut tendril = Tendril::new("SomeApp", "misc.txt");
        set_all_platform_paths(
            &mut tendril,
            &[given_parent_folder_1, given_parent_folder_2, PathBuf::from("I_Do_Not_Exist")]
        );

        pull_tendril(&given_tendrils_folder, &tendril).unwrap();

        let dest_file_contents = read_to_string(
            given_tendrils_folder.join("SomeApp").join("misc.txt")
        ).unwrap();
        assert_eq!(dest_file_contents, "Copy me!");
        assert!(given_tendrils_folder.join("SomeApp").read_dir().unwrap().count() == 1);
    }

    #[test]
    fn multiple_paths_first_is_missing_returns_not_found_error() {
        let temp_parent_folder = TempDir::new_in(
            get_disposable_folder(),
            "ParentFolder"
        ).unwrap().into_path();
        let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
        let source = temp_parent_folder.join("misc.txt");
        write(&source, "Source file contents").unwrap();

        let mut tendril = Tendril::new("SomeApp", "misc.txt");
        set_all_platform_paths(
            &mut tendril,
            &[PathBuf::from("I_Do_Not_Exist"), source]
        );

        let actual = pull_tendril(&given_tendrils_folder, &tendril);

        match actual {
            Err(PushPullError::IoError(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::NotFound)
            },
            _ => panic!(),
        }
        assert!(is_empty(given_tendrils_folder));
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
