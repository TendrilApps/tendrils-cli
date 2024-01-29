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

pub fn get_disposable_folder() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("temp-tendrils-folders");

    std::fs::create_dir_all(&path).unwrap();
    path
}

pub fn get_samples_folder() -> PathBuf {
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

pub struct Setup {
    pub temp_dir: TempDir, // Must return a reference to keep it in scope
    pub parent_dir: PathBuf,
    pub tendrils_dir: PathBuf,
    pub source_file: PathBuf,
    pub source_folder: PathBuf,
    pub dest_file: PathBuf,
    pub dest_folder: PathBuf,
    pub dest_nested_file: PathBuf,
    pub tendril: ResolvedTendril,
}

impl Setup {
    /// Create a new temporary test folder setup
    pub fn new(opts: &SetupOpts) -> Setup {
        let temp_dir: TempDir;
        let parent_dir: PathBuf;
        if opts.parent_dir_is_child_of_temp_dir {
            temp_dir = TempDir::new_in(
                get_disposable_folder(),
                "GrandparentFolder",
            ).unwrap();
            parent_dir = temp_dir.path().join(opts.parent_dirname).to_owned();
            create_dir_all(&parent_dir).unwrap();
        }
        else {
            temp_dir = TempDir::new_in(
                get_disposable_folder(),
                opts.parent_dirname,
            ).unwrap();
            parent_dir = temp_dir.path().to_owned();
        }
        let tendrils_dir = temp_dir.path().join(opts.tendrils_dirname);
        let source_file = parent_dir.join(opts.source_filename);
        let source_folder = parent_dir.join(opts.source_foldername);
        let source_nested_file = source_folder.join("nested.txt");
        let dest_file = tendrils_dir.join(opts.group).join(opts.source_filename);
        let dest_folder = tendrils_dir.join(opts.group).join(opts.source_foldername);
        let dest_nested_file = dest_folder.join("nested.txt");
        let tendril = match opts.is_folder_tendril {
            false => ResolvedTendril::new(opts.group.to_string(), opts.source_filename.to_string(), parent_dir.clone(), TendrilMode::FolderOverwrite),
            true => ResolvedTendril::new(opts.group.to_string(), opts.source_foldername.to_string(), parent_dir.clone(), TendrilMode::FolderOverwrite),
        }.unwrap();

        if opts.make_source_file {
            write(&source_file, "Source file contents").unwrap();
        }
        if opts.make_source_folder {
            create_dir_all(&source_folder).unwrap();

            if opts.make_source_nested_file {
                write(&source_nested_file, "Nested file contents").unwrap();
            }
        }
        if opts.make_tendrils_folder {
            create_dir_all(&tendrils_dir).unwrap();
        }

        Setup {
            temp_dir,
            parent_dir,
            tendrils_dir,
            source_file,
            source_folder,
            dest_file,
            dest_folder,
            dest_nested_file,
            tendril,
        }
    }

    pub fn dest_file_contents(&self) -> String {
        read_to_string(&self.dest_file).unwrap()
    }

    pub fn dest_nested_file_contents(&self) -> String {
        read_to_string(&self.dest_nested_file).unwrap()
    }

    pub fn source_file_contents(&self) -> String {
        read_to_string(&self.source_file).unwrap()
    }
}

pub struct SetupOpts<'a> {
    pub make_source_file: bool,
    pub make_source_folder: bool,
    pub make_source_nested_file: bool,
    pub make_tendrils_folder: bool,
    /// Sets the tendril to the misc folder, otherwise
    /// it is set to misc.txt
    pub is_folder_tendril: bool,
    pub parent_dir_is_child_of_temp_dir: bool,
    pub parent_dirname: &'a str,
    pub group: &'a str,
    pub source_filename: &'a str,
    pub source_foldername: &'a str,
    pub tendrils_dirname: &'a str,
}

impl<'a> SetupOpts<'a> {
    pub fn default() -> SetupOpts<'a> {
        SetupOpts {
            make_source_file: true,
            make_source_folder: true,
            make_source_nested_file: true,
            make_tendrils_folder: false,
            is_folder_tendril: false,
            parent_dir_is_child_of_temp_dir: false,
            parent_dirname: "ParentFolder",
            group: "SomeApp",
            source_filename: "misc.txt",
            source_foldername: "misc",
            tendrils_dirname: "TendrilsFolder",
        }
    }
}
