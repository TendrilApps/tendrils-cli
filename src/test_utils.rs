use crate::Tendril;
use crate::resolved_tendril::{
    ResolvedTendril,
    TendrilMode,
};
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

pub fn get_username_can_panic() -> String {
    match std::env::consts::OS {
        "macos" => std::env::var("USER").unwrap(),
        "windows" => std::env::var("USERNAME").unwrap(),
        _ => unimplemented!()
    }
}

pub fn get_mut_testing_var() -> Result<String, std::env::VarError> {
    std::env::var("mut-testing")
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

pub fn set_all_platform_paths(tendril: &mut Tendril, paths: &[PathBuf]) {
    let path_strings:Vec<String> = paths
        .iter()
        .map(|x| x.to_str().unwrap().to_string())
        .collect();

    tendril.parent_dirs_mac = path_strings.clone();
    tendril.parent_dirs_windows = path_strings;
}

pub struct Setup {
    pub temp_dir: TempDir, // Must return a reference to keep it in scope
    pub parent_dir: PathBuf,
    pub tendrils_dir: PathBuf,
    pub local_file: PathBuf,
    pub local_dir: PathBuf,
    pub ctrl_file: PathBuf,
    pub ctrl_dir: PathBuf,
    pub ctrl_nested_file: PathBuf,
    pub tendril: ResolvedTendril,
}

impl Setup {
    /// Create a new temporary test folder setup
    pub fn new(opts: &SetupOpts) -> Setup {
        let temp_dir: TempDir;
        let parent_dir: PathBuf;
        if opts.parent_dir_is_child_of_temp_dir {
            temp_dir = TempDir::new_in(
                get_disposable_dir(),
                "GrandparentDir",
            ).unwrap();
            parent_dir = temp_dir.path().join(opts.parent_dirname).to_owned();
            create_dir_all(&parent_dir).unwrap();
        }
        else {
            temp_dir = TempDir::new_in(
                get_disposable_dir(),
                opts.parent_dirname,
            ).unwrap();
            parent_dir = temp_dir.path().to_owned();
        }
        let tendrils_dir = temp_dir.path().join(opts.tendrils_dirname);
        let local_file = parent_dir.join(opts.local_filename);
        let local_dir = parent_dir.join(opts.local_dirname);
        let local_nested_file = local_dir.join("nested.txt");
        let ctrl_file = tendrils_dir.join(opts.group).join(opts.local_filename);
        let ctrl_dir = tendrils_dir.join(opts.group).join(opts.local_dirname);
        let ctrl_nested_file = ctrl_dir.join("nested.txt");
        let tendril = match opts.is_dir_tendril {
            false => ResolvedTendril::new(opts.group.to_string(), opts.local_filename.to_string(), parent_dir.clone(), TendrilMode::DirOverwrite),
            true => ResolvedTendril::new(opts.group.to_string(), opts.local_dirname.to_string(), parent_dir.clone(), TendrilMode::DirOverwrite),
        }.unwrap();

        if opts.make_local_file {
            write(&local_file, "Source file contents").unwrap();
        }
        if opts.make_local_dir {
            create_dir_all(&local_dir).unwrap();

            if opts.make_local_nested_file {
                write(&local_nested_file, "Nested file contents").unwrap();
            }
        }
        if opts.make_tendrils_dir {
            create_dir_all(&tendrils_dir).unwrap();
        }

        Setup {
            temp_dir,
            parent_dir,
            tendrils_dir,
            local_file,
            local_dir,
            ctrl_file,
            ctrl_dir,
            ctrl_nested_file,
            tendril,
        }
    }

    pub fn ctrl_file_contents(&self) -> String {
        read_to_string(&self.ctrl_file).unwrap()
    }

    pub fn ctrl_nested_file_contents(&self) -> String {
        read_to_string(&self.ctrl_nested_file).unwrap()
    }

    pub fn local_file_contents(&self) -> String {
        read_to_string(&self.local_file).unwrap()
    }
}

pub struct SetupOpts<'a> {
    pub make_tendrils_dir: bool,
    pub make_local_file: bool,
    pub make_local_dir: bool,
    pub make_local_nested_file: bool,
    /// Sets the tendril to the misc folder, otherwise
    /// it is set to misc.txt
    pub is_dir_tendril: bool,
    pub parent_dir_is_child_of_temp_dir: bool,
    pub parent_dirname: &'a str,
    pub group: &'a str,
    pub local_filename: &'a str,
    pub local_dirname: &'a str,
    pub tendrils_dirname: &'a str,
}

impl<'a> SetupOpts<'a> {
    pub fn default() -> SetupOpts<'a> {
        SetupOpts {
            make_tendrils_dir: false,
            make_local_file: true,
            make_local_dir: true,
            make_local_nested_file: true,
            is_dir_tendril: false,
            parent_dir_is_child_of_temp_dir: false,
            parent_dirname: "ParentDir",
            group: "SomeApp",
            local_filename: "misc.txt",
            local_dirname: "misc",
            tendrils_dirname: "TendrilsDir",
        }
    }
}
