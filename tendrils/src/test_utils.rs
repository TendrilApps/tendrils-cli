use crate::{
    batch_tendril_action_updating,
    symlink,
    ActionMode,
    ActionLog,
    FilterSpec,
    Fso,
    InitError,
    SetupError,
    Tendril,
    TendrilBundle,
    TendrilMode,
    TendrilReport,
    TendrilsApi,
};
use crate::config::{Config, parse_config};
use std::fs::{create_dir_all, read_to_string, write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempdir::TempDir;

/// Same behaviour as `batch_tendril_action_updating` except reports are only
/// returned once all actions have completed.
/// Easier API for testing.
pub fn batch_tendril_action(
    mode: ActionMode,
    td_dir: &Path,
    td_bundles: Vec<TendrilBundle>,
    dry_run: bool,
    force: bool,
) -> Vec<TendrilReport<ActionLog>> {
    let mut reports = vec![];
    let updater = |r| reports.push(r);

    batch_tendril_action_updating(updater, mode, td_dir, td_bundles, dry_run, force);
    reports
}

pub fn get_disposable_dir() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target")
        .join("tempdirs");

    std::fs::create_dir_all(&path).unwrap();
    path
}

pub fn get_samples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("samples")
}

pub fn is_empty(dir: &Path) -> bool {
    if dir.exists() {
        if !dir.is_dir() {
            panic!("Expected a folder")
        }
        return dir.read_dir().unwrap().count() == 0;
    }
    true
}

/// Exposes the otherwise private function
pub fn parse_config_expose(
    json: &str,
) -> Result<Config, serde_json::Error> {
    parse_config(json)
}

pub fn set_parents(tendril: &mut TendrilBundle, paths: &[PathBuf]) {
    let path_strings: Vec<String> =
        paths.iter().map(|x| String::from(x.to_str().unwrap())).collect();

    tendril.parents = path_strings;
}

/// Exposes the otherwise private function
pub fn symlink_expose(
    create_at: &Path,
    target: &Path,
    dry_run: bool,
    force: bool,
) -> Result<crate::TendrilActionSuccess, crate::TendrilActionError> {
    symlink(
        create_at,
        &create_at.get_type(),
        target,
        &target.get_type(),
        dry_run,
        force,
    )
}

#[cfg(windows)]
fn get_username() -> String {
    std::env::var("USERNAME").unwrap()
}

/// File or folder must already exist
pub fn set_ra(path: &Path, can_read: bool) {
    #[cfg(windows)]
    {
        if can_read {
            let mut cmd = Command::new("ICACLS");
            let output = cmd
                .arg(path)
                .arg("/grant")
                .arg(format!("{}:(RX)", get_username()))
                .output()
                .unwrap();
            if !output.status.success() {
                let err = format!("ICACLS command failed: {:?}", output);
                println!("{err}");
            }
        }
        else {
            let mut cmd = Command::new("ICACLS");
            let output = cmd
                .arg(path)
                .arg("/inheritance:r")
                .output().unwrap();
            if !output.status.success() {
                let err = format!("ICACLS command failed: {:?}", output);
                println!("{err}");
            }

            let mut cmd = Command::new("ICACLS");
            let output = cmd
                .arg(path)
                .arg("/grant")
                .arg(format!("{}:(W)", get_username()))
                .output()
                .unwrap();
            if !output.status.success() {
                let err = format!("ICACLS command failed: {:?}", output);
                println!("{err}");
            }
        }
    }
    #[cfg(not(windows))]
    {
        if can_read {
            let mut cmd = Command::new("chmod");
            let output = cmd
                .arg("u+rw")
                .arg(path)
                .output().unwrap();
            if !output.status.success() {
                let err = format!("chmod command failed: {:?}", output);
                println!("{err}");
            }
        }
        else {
            let mut cmd = Command::new("chmod");
            let output = cmd
                .arg("u-rw")
                .arg(path)
                .output().unwrap();
            if !output.status.success() {
                let err = format!("chmod command failed: {:?}", output);
                println!("{err}");
            }
        }
    }
}

pub struct MockTendrilsApi<'a> {
    pub init_const_rt: Result<(), InitError>,
    pub init_fn: Option<Box<dyn Fn(&Path, bool) -> Result<(), InitError>>>,
    pub init_exp_dir_arg: PathBuf,
    pub init_exp_force_arg: bool,
    pub is_tendrils_dir_const_rt: bool,
    pub is_tendrils_dir_fn: Option<Box<dyn Fn(&Path) -> bool>>,
    pub tau_const_updater_rts: Vec<TendrilReport<ActionLog>>,
    pub tau_const_rt: Result<(), SetupError>,
    pub ta_const_rt: Result<Vec<TendrilReport<ActionLog>>, SetupError>,
    pub ta_fn: Option<
        Box<
            dyn Fn(
                ActionMode,
                Option<&Path>,
                FilterSpec,
                bool,
                bool,
            )
                -> Result<Vec<TendrilReport<ActionLog>>, SetupError>,
        >,
    >,
    pub ta_exp_mode: ActionMode,
    pub ta_exp_path: Option<&'a Path>,
    pub ta_exp_filter: FilterSpec<'a>,
    pub ta_exp_dry_run: bool,
    pub ta_exp_force: bool,
}

impl<'a> MockTendrilsApi<'a> {
    pub fn new() -> MockTendrilsApi<'a> {
        MockTendrilsApi {
            init_const_rt: Ok(()),
            init_exp_dir_arg: PathBuf::from(""),
            init_exp_force_arg: false,
            init_fn: None,
            is_tendrils_dir_const_rt: true,
            is_tendrils_dir_fn: None,
            tau_const_updater_rts: vec![],
            tau_const_rt: Ok(()),
            ta_const_rt: Ok(vec![]),
            ta_fn: None,
            ta_exp_mode: ActionMode::Pull,
            ta_exp_path: None,
            ta_exp_filter: FilterSpec::new(),
            ta_exp_dry_run: false,
            ta_exp_force: false,
        }
    }
}

impl TendrilsApi for MockTendrilsApi<'_> {
    fn init_tendrils_dir(
        &self,
        dir: &Path,
        force: bool,
    ) -> Result<(), InitError> {
        assert_eq!(dir, self.init_exp_dir_arg);
        assert_eq!(force, self.init_exp_force_arg);

        if let Some(f) = self.init_fn.as_ref() {
            f(dir, force)
        }
        else {
            self.init_const_rt.clone()
        }
    }

    fn is_tendrils_dir(&self, dir: &Path) -> bool {
        if let Some(f) = self.is_tendrils_dir_fn.as_ref() {
            f(dir)
        }
        else {
            self.is_tendrils_dir_const_rt
        }
    }

    fn tendril_action_updating<F: FnMut(TendrilReport<ActionLog>)>(
        &self,
        mut update_fn: F,
        _: ActionMode,
        _: Option<&Path>,
        _: FilterSpec,
        _: bool,
        _: bool,
    ) -> Result<(), SetupError> {
        for update_rt in self.tau_const_updater_rts.iter() {
            update_fn(update_rt.clone());
        }

        self.tau_const_rt.clone()
    }

    fn tendril_action(
        &self,
        mode: ActionMode,
        td_dir: Option<&Path>,
        filter: FilterSpec,
        dry_run: bool,
        force: bool,
    ) -> Result<Vec<TendrilReport<ActionLog>>, SetupError> {
        assert_eq!(mode, self.ta_exp_mode);
        assert_eq!(td_dir, self.ta_exp_path);
        assert_eq!(filter, self.ta_exp_filter);
        assert_eq!(dry_run, self.ta_exp_dry_run);
        assert_eq!(force, self.ta_exp_force);

        if let Some(f) = self.ta_fn.as_ref() {
            f(mode, td_dir, filter, dry_run, force)
        }
        else {
            self.ta_const_rt.clone()
        }
    }
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
    pub remote_subdir_file: PathBuf,
    pub remote_subdir_dir: PathBuf,
    pub remote_subdir_nested_file: PathBuf,
    pub local_file: PathBuf,
    pub local_dir: PathBuf,
    pub local_nested_file: PathBuf,
    pub local_subdir_file: PathBuf,
    pub local_subdir_dir: PathBuf,
    pub local_subdir_nested_file: PathBuf,
    pub target_file: PathBuf,
    pub target_dir: PathBuf,
    pub target_nested_file: PathBuf,
    pub remote_nra_file: PathBuf,
    pub remote_nra_dir: PathBuf,
    pub remote_nra_nested_file: PathBuf,
    pub local_nra_file: PathBuf,
    pub local_nra_dir: PathBuf,
    pub local_nra_nested_file: PathBuf,
}

impl Default for Setup {
    fn default() -> Self {
        Self::new()
    }
}

impl Setup {
    /// Create a new temporary test folder setup
    pub fn new() -> Setup {
        let temp_dir =
            TempDir::new_in(get_disposable_dir(), "ParentDir").unwrap();
        let parent_dir = temp_dir.path().to_owned();
        let td_dir = temp_dir.path().join("TendrilsDir");
        let td_json_file = td_dir.join("tendrils.json");
        let group_dir = td_dir.join("SomeApp");
        let remote_file = parent_dir.join("misc.txt");
        let remote_dir = parent_dir.join("misc");
        let remote_nested_file = remote_dir.join("nested.txt");
        let remote_subdir_file = parent_dir.join("SubDir").join("misc.txt");
        let remote_subdir_dir = parent_dir.join("SubDir").join("misc");
        let remote_subdir_nested_file = remote_subdir_dir.join("nested.txt");
        let local_file = group_dir.join("misc.txt");
        let local_dir = group_dir.join("misc");
        let local_nested_file = local_dir.join("nested.txt");
        let local_subdir_file = group_dir.join("SubDir").join("misc.txt");
        let local_subdir_dir = group_dir.join("SubDir").join("misc");
        let local_subdir_nested_file = local_subdir_dir.join("nested.txt");
        let target_file = parent_dir.join("target.txt");
        let target_dir = parent_dir.join("target");
        let target_nested_file = target_dir.join("nested.txt");
        let remote_nra_file = parent_dir.join("nra.txt");
        let remote_nra_dir = parent_dir.join("nra");
        let remote_nra_nested_file = remote_nra_dir.join("misc.txt");
        let local_nra_file = group_dir.join("nra.txt");
        let local_nra_dir = group_dir.join("nra");
        let local_nra_nested_file = remote_nra_dir.join("misc.txt");

        Setup {
            temp_dir,
            parent_dir,
            td_dir,
            td_json_file,
            group_dir,
            remote_file,
            remote_dir,
            remote_nested_file,
            remote_subdir_file,
            remote_subdir_dir,
            remote_subdir_nested_file,
            local_file,
            local_dir,
            local_nested_file,
            local_subdir_file,
            local_subdir_dir,
            local_subdir_nested_file,
            target_file,
            target_dir,
            target_nested_file,
            remote_nra_file,
            remote_nra_dir,
            remote_nra_nested_file,
            local_nra_file,
            local_nra_dir,
            local_nra_nested_file,
        }
    }

    pub fn file_tendril_bundle(&self) -> TendrilBundle {
        let mut bundle = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
        bundle.parents = vec![self.parent_dir.to_string_lossy().to_string()];
        bundle
    }

    pub fn file_tendril(&self) -> Tendril {
        Tendril::new(
            "SomeApp",
            "misc.txt",
            self.parent_dir.clone(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()
    }

    pub fn dir_tendril(&self) -> Tendril {
        Tendril::new(
            "SomeApp",
            "misc",
            self.parent_dir.clone(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()
    }

    pub fn subdir_file_tendril(&self) -> Tendril {
        Tendril::new(
            "SomeApp",
            "SubDir/misc.txt",
            self.parent_dir.clone(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()
    }

    pub fn subdir_dir_tendril(&self) -> Tendril {
        Tendril::new(
            "SomeApp",
            "SubDir/misc",
            self.parent_dir.clone(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()
    }

    pub fn make_parent_dir(&self) {
        create_dir_all(&self.parent_dir).unwrap();
    }

    pub fn make_td_dir(&self) {
        create_dir_all(&self.td_dir).unwrap();
    }

    pub fn make_td_json_file(&self, tendrils: &[TendrilBundle]) {
        self.make_td_dir();
        let config = Config { tendrils: tendrils.to_vec() };
        let json = serde_json::to_string(&config).unwrap();
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

    pub fn make_local_subdir_file(&self) {
        create_dir_all(self.group_dir.join("SubDir")).unwrap();
        write(&self.local_subdir_file, "Local subdir file contents").unwrap();
    }

    pub fn make_local_subdir_dir(&self) {
        create_dir_all(&self.local_subdir_dir).unwrap();
    }

    pub fn make_local_subdir_nested_file(&self) {
        self.make_local_subdir_dir();
        write(
            &self.local_subdir_nested_file,
            "Local subdir nested file contents",
        )
        .unwrap();
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

    pub fn make_remote_subdir_file(&self) {
        create_dir_all(self.parent_dir.join("SubDir")).unwrap();
        write(&self.remote_subdir_file, "Remote subdir file contents").unwrap();
    }

    pub fn make_remote_subdir_dir(&self) {
        create_dir_all(&self.remote_subdir_dir).unwrap();
    }

    pub fn make_remote_subdir_nested_file(&self) {
        self.make_remote_subdir_dir();
        write(
            &self.remote_subdir_nested_file,
            "Remote subdir nested file contents",
        )
        .unwrap();
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

    pub fn local_subdir_file_contents(&self) -> String {
        read_to_string(&self.local_subdir_file).unwrap()
    }

    pub fn local_subdir_nested_file_contents(&self) -> String {
        read_to_string(&self.local_subdir_nested_file).unwrap()
    }

    pub fn remote_subdir_file_contents(&self) -> String {
        read_to_string(&self.remote_subdir_file).unwrap()
    }

    pub fn remote_subdir_nested_file_contents(&self) -> String {
        read_to_string(&self.remote_subdir_nested_file).unwrap()
    }

    /// Create a file without read access
    pub fn make_remote_nra_file(&self) {
        write(&self.remote_nra_file, "Remote file contents").unwrap();
        set_ra(&self.remote_nra_file, false);
    }

    pub fn make_remote_nra_dir(&self) {
        create_dir_all(&self.remote_nra_dir).unwrap();
        set_ra(&self.remote_nra_dir, false);
    }

    pub fn make_remote_nested_nra_file(&self) {
        create_dir_all(&self.remote_nra_dir).unwrap();
        write(&self.remote_nra_nested_file, "Remote nested file contents").unwrap();
        set_ra(&self.remote_nra_nested_file, false);
    }

    pub fn make_local_nra_file(&self) {
        create_dir_all(&self.group_dir).unwrap();
        write(&self.local_nra_file, "Local file contents").unwrap();
        set_ra(&self.local_nra_file, false);
    }

    pub fn make_local_nra_dir(&self) {
        create_dir_all(&self.local_nra_dir).unwrap();
        set_ra(&self.local_nra_dir, false);
    }

    pub fn make_local_nested_nra_file(&self) {
        create_dir_all(&self.local_nra_dir).unwrap();
        write(&self.local_nra_nested_file, "Local nested file contents").unwrap();
        create_dir_all(&self.local_nra_nested_file).unwrap();
    }
}
