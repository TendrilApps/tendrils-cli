use std::path::{Path, PathBuf};
mod file_system;
use file_system::{FsProvider, FsWrapper};
mod tendril;
use tendril::Tendril;

fn main() {
    let fs_wrapper = FsWrapper {};

    let tendrils_folder = get_tendrils_folder(&fs_wrapper)
        .expect("Could not find a Tendrils folder");

    let common_tendrils = get_tendrils(&tendrils_folder, &fs_wrapper)
        .expect("Could not import the tendrils.json file");

    let override_tendrils = get_tendril_overrides(&tendrils_folder, &fs_wrapper)
        .expect("Could not import the tendrils-overrides.json file");

    if override_tendrils.is_empty() { println!("No local overrides were found.") }

    let resolved_tendrils = resolve_overrides(&common_tendrils, &override_tendrils);

    print!("{:#?}", resolved_tendrils);
}

fn get_tendrils_folder(fs: &impl FsProvider) -> Option<PathBuf> {
    let mut path = fs.current_dir().ok()?;
    path.push("tendrils.json");

    if fs.exists(&path) {
        path.pop();
        Some(path)
    } else {
        None
    }
}

fn get_tendrils(tendrils_folder: &Path, fs: &impl FsProvider) -> Result<Vec<Tendril>, GetTendrilsError> {
    let tendrils_file_path = Path::new(&tendrils_folder).join("tendrils.json");
    let tendrils_file_contents = fs.read_to_string(&tendrils_file_path)?;
    let tendrils = parse_tendrils(&tendrils_file_contents)?;
    Ok(tendrils)
}

fn get_tendril_overrides(tendrils_folder: &Path, fs: &impl FsProvider) -> Result<Vec<Tendril>, GetTendrilsError> {
    let tendrils_file_path = Path::new(&tendrils_folder).join("tendrils-overrides.json");
    let tendrils_file_contents: String;
    if tendrils_file_path.exists() {
        tendrils_file_contents = fs.read_to_string(&tendrils_file_path)?;
    } else {
        return Ok([].to_vec())
    }

    let tendrils = parse_tendrils(&tendrils_file_contents)?;
    Ok(tendrils)
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
fn resolve_overrides(global: &Vec<Tendril>, overrides: &Vec<Tendril>) -> Vec<Tendril> {
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
enum GetTendrilsError {
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
mod mocks {
    use std::{path::{ Path, PathBuf }, cell::RefCell};
    use super::file_system::FsProvider;

    pub const DEFAULT_CURRENT_DIR: &str = "DefaultCurrentDir";
    pub const DEFAULT_READ_TO_STRING: &str = "DefaultFileContents";

    pub struct FsWrapper {
        pub current_dir_return: Result<PathBuf, std::io::Error>,
        pub exists_returns: Vec<bool>,
        pub exists_queries: RefCell<Vec<PathBuf>>,
        pub read_to_string_return: Result<String, std::io::Error>
    }

    impl FsWrapper {
        pub fn new() -> FsWrapper {
            FsWrapper {
                current_dir_return: Ok(PathBuf::from(DEFAULT_CURRENT_DIR)),
                exists_returns: [false].to_vec(),
                exists_queries: RefCell::from([].to_vec()),
                read_to_string_return: Ok(DEFAULT_READ_TO_STRING.to_owned())
            }
        }
    }

    impl FsProvider for FsWrapper {
        fn current_dir(&self) -> Result<PathBuf, std::io::Error> {
            if self.current_dir_return.is_ok() {
                let value = self.current_dir_return.as_ref().unwrap();
                Ok(value.to_owned())
            } else {
                let errorkind = self.current_dir_return.as_ref().unwrap_err().kind();
                let err = std::io::Error::from(errorkind);
                Err(err)
            }
        }

        fn exists(&self, path: &Path) -> bool {
            self.exists_queries.borrow_mut().push(path.to_owned());
            self.exists_returns[self.exists_queries.borrow().len() - 1]
        }

        fn read_to_string(&self, _path: &Path) -> Result<String, std::io::Error> {
            if self.read_to_string_return.is_ok() {
                let value = self.read_to_string_return.as_ref().unwrap();
                Ok(value.to_owned())
            } else {
                let errorkind = self.read_to_string_return.as_ref().unwrap_err().kind();
                let err = std::io::Error::from(errorkind);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod get_tendrils_folder_tests {
    use std::cell::RefCell;
    use std::path::PathBuf;
    use super::get_tendrils_folder;
    use super::mocks::FsWrapper;

    #[test]
    fn no_tendrils_json_in_current_dir_returns_none() {
        let fs_mock = FsWrapper::new();
        let expected_exists_queries = [PathBuf::from(super::mocks::DEFAULT_CURRENT_DIR)
            .join("tendrils.json")].to_vec();

        let actual = get_tendrils_folder(&fs_mock);
        assert!(actual.is_none());
        assert_eq!(fs_mock.exists_queries, RefCell::from(expected_exists_queries));
    }

    #[test]
    fn tendrils_json_in_current_dir_returns_current_dir() {
        let mut fs_mock = FsWrapper::new();
        fs_mock.exists_returns = [true].to_vec();
        let expected = PathBuf::from(super::mocks::DEFAULT_CURRENT_DIR);

        let actual = get_tendrils_folder(&fs_mock);
        assert_eq!(actual.unwrap(), expected);
    }
}

#[cfg(test)]
mod parse_tendrils_tests {
    use super::parse_tendrils;

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
    fn json_missing_field_returns_error() {
        let given = r#"
        [
            {
                "app": "MyApp",
                "parent-dirs-mac": [],
                "parent-dirs-windows": ["C:\\Users\\<user>\\AppData\\"],
                "folder-merge": false
            }
        ]"#;

        assert!(parse_tendrils(&given).is_err());
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
        let given = r#"
        [
            {
                "app": "MyApp",
                "name": "settings.json",
                "parent-dirs-mac": [],
                "parent-dirs-windows": ["C:\\Users\\<user>\\AppData\\"],
                "folder-merge": false
            }
        ]"#;

        let tendril1 = super::Tendril {
            app: "MyApp".to_string(),
            name: "settings.json".to_string(),
            parent_dirs_mac: [].to_vec(),
            parent_dirs_windows: ["C:\\Users\\<user>\\AppData\\".to_string()].to_vec(),
            folder_merge: false,
        };

        let expected = [tendril1].to_vec();

        let actual = parse_tendrils(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn multiple_tendrils_in_json_returns_tendrils() {
        let given = r#"
        [
            {
                "app": "MyApp",
                "name": "settings.json",
                "parent-dirs-mac": [],
                "parent-dirs-windows": ["C:\\Users\\<user>\\AppData\\"],
                "folder-merge": false
            },
            {
                "app": "MyApp2",
                "name": "settings2.json",
                "parent-dirs-mac": ["some/mac/path"],
                "parent-dirs-windows": ["C:\\Users\\<user>\\Documents\\"],
                "folder-merge": true
            }
        ]"#;

        let tendril1 = super::Tendril {
            app: "MyApp".to_string(),
            name: "settings.json".to_string(),
            parent_dirs_mac: [].to_vec(),
            parent_dirs_windows: ["C:\\Users\\<user>\\AppData\\".to_string()].to_vec(),
            folder_merge: false,
        };

        let tendril2 = super::Tendril {
            app: "MyApp2".to_string(),
            name: "settings2.json".to_string(),
            parent_dirs_mac: ["some/mac/path".to_string()].to_vec(),
            parent_dirs_windows: ["C:\\Users\\<user>\\Documents\\".to_string()].to_vec(),
            folder_merge: true,
        };

        let expected = [tendril1, tendril2].to_vec();

        let actual = parse_tendrils(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn ignores_extra_json_field_returns_tendril() {
        let given = r#"
        [
            {
                "some-extra-field": "ABCD",
                "app": "MyApp",
                "name": "settings.json",
                "parent-dirs-mac": [],
                "parent-dirs-windows": ["C:\\Users\\<user>\\AppData\\"],
                "folder-merge": false
            }
        ]"#;

        let tendril1 = super::Tendril {
            app: "MyApp".to_string(),
            name: "settings.json".to_string(),
            parent_dirs_mac: [].to_vec(),
            parent_dirs_windows: ["C:\\Users\\<user>\\AppData\\".to_string()].to_vec(),
            folder_merge: false,
        };

        let expected = [tendril1].to_vec();
        let actual = parse_tendrils(&given).unwrap();

        assert_eq!(actual, expected);
    }
}
