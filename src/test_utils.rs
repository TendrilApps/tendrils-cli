use crate::resolved_tendril::ResolvedTendril;
use crate::{Tendril, TendrilMode};
use std::fs::{
    create_dir_all,
    read_to_string,
    write,
};
use std::path::{Path, PathBuf};
use tempdir::TempDir;

pub fn get_disposable_dir() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("tempdirs");

    std::fs::create_dir_all(&path).unwrap();
    path
}

pub fn get_samples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("samples")
}

pub fn is_empty(dir: &Path) -> bool {
    if dir.exists() {
        if !dir.is_dir() {
            panic!("Expected a folder")
        }
        return dir.read_dir().unwrap().count() == 0
    }
    true
}

pub fn set_parents(tendril: &mut Tendril, paths: &[PathBuf]) {
    let path_strings:Vec<String> = paths
        .iter()
        .map(|x| x.to_str().unwrap().to_string())
        .collect();

    tendril.parents = path_strings;
}

pub struct Setup {
    pub temp_dir: TempDir, // Must return a reference to keep it in scope
    pub parent_dir: PathBuf,
    pub td_dir: PathBuf,
    pub td_json_file: PathBuf,
    pub group_dir: PathBuf,
    pub remote_file: PathBuf,
    pub remote_dir: PathBuf,
    pub remote_nested_file: PathBuf,
    pub local_file: PathBuf,
    pub local_dir: PathBuf,
    pub local_nested_file: PathBuf,
    pub target_file: PathBuf,
    pub target_dir: PathBuf,
    pub target_nested_file: PathBuf,
}

impl Setup {
    /// Create a new temporary test folder setup
    pub fn new() -> Setup {
        let temp_dir = TempDir::new_in(
            get_disposable_dir(),
            "ParentDir",
        ).unwrap();
        let parent_dir = temp_dir.path().to_owned();
        let td_dir = temp_dir.path().join("TendrilsDir");
        let td_json_file = td_dir.join("tendrils.json");
        let group_dir = td_dir.join("SomeApp");
        let remote_file = parent_dir.join("misc.txt");
        let remote_dir = parent_dir.join("misc");
        let remote_nested_file = remote_dir.join("nested.txt");
        let local_file = group_dir.join("misc.txt");
        let local_dir = group_dir.join("misc");
        let local_nested_file = local_dir.join("nested.txt");
        let target_file = parent_dir.join("target.txt");
        let target_dir = parent_dir.join("target");
        let target_nested_file = target_dir.join("nested.txt");

        Setup {
            temp_dir,
            parent_dir,
            td_dir,
            td_json_file,
            group_dir,
            remote_file,
            remote_dir,
            remote_nested_file,
            local_file,
            local_dir,
            local_nested_file,
            target_file,
            target_dir,
            target_nested_file,
        }
    }

    pub fn file_tendril(&self) -> Tendril {
        Tendril::new(
            "SomeApp",
            vec!["misc.txt"],
        )
    }

    pub fn resolved_file_tendril(&self) -> ResolvedTendril {
        ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            self.parent_dir.clone(),
            TendrilMode::DirOverwrite,
        ).unwrap()
    }

    pub fn resolved_dir_tendril(&self) -> ResolvedTendril {
        ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc".to_string(),
            self.parent_dir.clone(),
            TendrilMode::DirOverwrite,
        ).unwrap()
    }

    pub fn make_parent_dir(&self) {
        create_dir_all(&self.parent_dir).unwrap();
    }

    pub fn make_td_dir(&self) {
        create_dir_all(&self.td_dir).unwrap();
    }

    pub fn make_td_json_file(&self, tendrils: &[Tendril]) {
        self.make_td_dir();
        let json = serde_json::to_string(tendrils).unwrap();
        write(&self.td_json_file, json).unwrap();
    }

    pub fn make_group_dir(&self) {
        create_dir_all(&self.group_dir).unwrap();
    }

    pub fn make_local_file(&self) {
        create_dir_all(&self.group_dir).unwrap();
        write(&self.local_file, "Local file contents").unwrap();
    }

    pub fn make_local_dir(&self) {
        create_dir_all(&self.local_dir).unwrap();
    }

    pub fn make_local_nested_file(&self) {
        self.make_local_dir();
        write(&self.local_nested_file, "Local nested file contents").unwrap();
    }

    pub fn make_remote_file(&self) {
        write(&self.remote_file, "Remote file contents").unwrap();
    }

    pub fn make_remote_dir(&self) {
        create_dir_all(&self.remote_dir).unwrap();
    }

    pub fn make_remote_nested_file(&self) {
        self.make_remote_dir();
        write(&self.remote_nested_file, "Remote nested file contents").unwrap();
    }

    pub fn make_target_file(&self) {
        write(&self.target_file, "Target file contents").unwrap();
    }

    pub fn make_target_dir(&self) {
        create_dir_all(&self.target_dir).unwrap();
    }

    pub fn make_target_nested_file(&self) {
        self.make_target_dir();
        write(&self.target_nested_file, "Target nested file contents").unwrap();
    }

    pub fn local_file_contents(&self) -> String {
        read_to_string(&self.local_file).unwrap()
    }

    pub fn local_nested_file_contents(&self) -> String {
        read_to_string(&self.local_nested_file).unwrap()
    }

    pub fn remote_file_contents(&self) -> String {
        read_to_string(&self.remote_file).unwrap()
    }

    pub fn remote_nested_file_contents(&self) -> String {
        read_to_string(&self.remote_nested_file).unwrap()
    }
}
