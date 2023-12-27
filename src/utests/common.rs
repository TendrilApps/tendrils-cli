use std::path::{Path, PathBuf};
use crate::Tendril;

pub fn get_disposable_folder() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("temp-tendrils-folders");

    if !path.is_dir() {
        std::fs::create_dir(&path).unwrap();
    }
    path
}

pub fn get_username() -> String {
    match std::env::consts::OS {
        "macos" => std::env::var("USER").unwrap(),
        "windows" => std::env::var("USERNAME").unwrap(),
        _ => unimplemented!()
    }
}

pub fn is_empty(folder: &Path) -> bool {
    if folder.exists() {
        if !folder.is_dir() {
            panic!("Expected a folder")
        }
        return folder.read_dir().unwrap().count() == 0
    }
    true
}

pub fn set_all_platform_paths(tendril: &mut Tendril, paths: &[PathBuf]) {
    let path_strings:Vec<String> = paths
        .iter()
        .map(|x| x.to_str().unwrap().to_string())
        .collect();

    tendril.parent_dirs_mac = path_strings.clone();
    tendril.parent_dirs_windows = path_strings;
}
