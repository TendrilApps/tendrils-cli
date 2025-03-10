use crate::{
    symlink,
    ActionMode,
    ActionLog,
    FilterSpec,
    InitError,
    PathExt,
    SetupError,
    RawTendril,
    Tendril,
    TendrilMode,
    TendrilReport,
    TendrilsApi,
    UniPath,
    UpdateHandler,
};
use crate::config::Config;
use crate::enums::GetConfigError;
use std::env::var;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempdir::TempDir;

// Serial test sets
pub const SERIAL_CD: &str = "cd";
pub const SERIAL_MUT_ENV_VARS: &str = "mut-env-var";
pub const SERIAL_ROOT: &str = "root";

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

pub fn home_dir() -> PathBuf {
    PathBuf::from(var("HOME").unwrap())
}

/// Path to the global `~/.tendrils` folder using the current value of
/// the `HOME` environment variable
pub fn global_cfg_dir() -> PathBuf {
    home_dir().join(".tendrils")
}

/// Path to the global `~/.tendrils/global-config.json` file using the
/// `.tendrils` folder defined in [`global_cfg_dir`]
pub fn global_cfg_file() -> PathBuf {
    global_cfg_dir().join("global-config.json")
}

/// Creates the json for a `global-config.json` file containing only a
/// `default-repo-path` field
pub fn default_repo_path_as_json(default_repo_path: &str) -> String {
    format!("{{\"default-repo-path\": \"{}\"}}", default_repo_path)
}

/// Returns "fo�o" where the third character is invalid UTF-8.
pub fn non_utf_8_text() -> std::ffi::OsString {
    #[cfg(unix)]
    {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        // Here, the values 0x66 and 0x6f correspond to 'f' and 'o'
        // respectively. The value 0x80 is a lone continuation byte, invalid
        // in a UTF-8 sequence.
        let source = [0x66, 0x6f, 0x80, 0x6f];
        OsStr::from_bytes(&source[..]).to_os_string()
    }
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::prelude::OsStringExt;

        // Here the values 0x0066 and 0x006f correspond to 'f' and 'o'
        // respectively. The value 0xD800 is a lone surrogate half, invalid
        // in a UTF-8 sequence.
        let source = [0x0066, 0x006f, 0xD800, 0x006f];
        OsString::from_wide(&source[..])
    }
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

/// Check if UAC is enabled. Useful when test are running in the context
/// of a Github Action, as UAC is disabled here.
pub fn uac_enabled() -> bool {
    #[cfg(windows)] {
        let mut cmd = std::process::Command::new("REG");
        let output = cmd
            .arg("QUERY")
            .arg("HKEY_LOCAL_MACHINE\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\System\\")
            .arg("/v")
            .arg("ConsentPromptBehaviorAdmin")
            .output()
            .unwrap();
        if !output.status.success() {
            panic!("ERROR: Could not check UAC status");
        }

        !String::from_utf8(output.stdout).unwrap().contains("0x0")
    }

    #[cfg(not(windows))] {
        true
    }
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
    pub init_fn: Option<Box<dyn Fn(&UniPath, bool) -> Result<(), InitError>>>,
    pub init_exp_dir_arg: PathBuf,
    pub init_exp_force_arg: bool,
    pub is_tendrils_repo_const_rt: bool,
    pub is_tendrils_repo_fn: Option<Box<dyn Fn(&UniPath) -> bool>>,
    pub get_default_repo_const_rt: Result<Option<PathBuf>, GetConfigError>,
    pub get_default_repo_fn: Option<Box<dyn Fn() -> Result<Option<PathBuf>, GetConfigError>>>,
    pub get_default_profiles_const_rt: Result<Option<Vec<String>>, GetConfigError>,
    pub get_default_profiles_fn: Option<Box<dyn Fn() -> Result<Option<Vec<String>>, GetConfigError>>>,
    pub tau_const_count_updater_rt: i32,
    pub tau_const_before_updater_rts: Vec<RawTendril>,
    pub tau_const_after_updater_rts: Vec<TendrilReport<ActionLog>>,
    pub tau_const_rt: Result<(), SetupError>,
    pub ta_const_rt: Result<Vec<TendrilReport<ActionLog>>, SetupError>,
    pub ta_fn: Option<
        Box<
            dyn Fn(
                ActionMode,
                Option<&UniPath>,
                FilterSpec,
                bool,
                bool,
            )
                -> Result<Vec<TendrilReport<ActionLog>>, SetupError>,
        >,
    >,
    pub ta_exp_mode: ActionMode,
    pub ta_exp_path: Option<&'a Path>,
    pub ta_exp_filter: FilterSpec,
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
            is_tendrils_repo_const_rt: true,
            is_tendrils_repo_fn: None,
            get_default_repo_const_rt: Ok(None),
            get_default_repo_fn: None,
            get_default_profiles_const_rt: Ok(None),
            get_default_profiles_fn: None,
            tau_const_count_updater_rt: 0,
            tau_const_before_updater_rts: vec![],
            tau_const_after_updater_rts: vec![],
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
    fn init_tendrils_repo(
        &self,
        dir: &UniPath,
        force: bool,
    ) -> Result<(), InitError> {
        assert_eq!(dir.inner(), self.init_exp_dir_arg);
        assert_eq!(force, self.init_exp_force_arg);

        if let Some(f) = self.init_fn.as_ref() {
            f(dir, force)
        }
        else {
            self.init_const_rt.clone()
        }
    }

    fn is_tendrils_repo(&self, dir: &UniPath) -> bool {
        if let Some(f) = self.is_tendrils_repo_fn.as_ref() {
            f(dir)
        }
        else {
            self.is_tendrils_repo_const_rt
        }
    }

    fn get_default_repo_path(&self) -> Result<Option<PathBuf>, GetConfigError> {
        if let Some(f) = self.get_default_repo_fn.as_ref() {
            f()
        }
        else {
            self.get_default_repo_const_rt.clone()
        }
    }

    fn get_default_profiles(&self) -> Result<Option<Vec<String>>, GetConfigError> {
        if let Some(f) = self.get_default_profiles_fn.as_ref() {
            f()
        }
        else {
            self.get_default_profiles_const_rt.clone()
        }
    }

    fn tendril_action_updating<U>(
        &self,
        mut updater: U,
        mode: ActionMode,
        td_repo: Option<&UniPath>,
        filter: FilterSpec,
        dry_run: bool,
        force: bool,
    ) -> Result<(), SetupError> 
    where
        U: UpdateHandler<ActionLog>
    {
        if self.tau_const_rt.is_err() {
            return self.tau_const_rt.clone();
        }

        assert_eq!(mode, self.ta_exp_mode);
        assert_eq!(filter, self.ta_exp_filter);
        assert_eq!(dry_run, self.ta_exp_dry_run);
        assert_eq!(force, self.ta_exp_force);
        if let Some(p) = td_repo {
            assert_eq!(p.inner(), self.ta_exp_path.unwrap());
        }
        else {
            assert_eq!(None, self.ta_exp_path);
        }
        assert_eq!(
            self.tau_const_count_updater_rt,
            self.tau_const_before_updater_rts.len() as i32,
            "Incorrect number of returns for 'before' callbacks",
        );
        assert_eq!(
            self.tau_const_count_updater_rt,
            self.tau_const_after_updater_rts.len() as i32,
            "Incorrect number of returns for 'after' callbacks",
        );

        updater.count(self.tau_const_count_updater_rt);
        for i in 0..self.tau_const_count_updater_rt {
            updater.before(self.tau_const_before_updater_rts[i as usize].clone());

            // =========================
            // Action would be done here
            // =========================

            updater.after(self.tau_const_after_updater_rts[i as usize].clone());
        }

        Ok(())
    }

    fn tendril_action(
        &self,
        mode: ActionMode,
        td_repo: Option<&UniPath>,
        filter: FilterSpec,
        dry_run: bool,
        force: bool,
    ) -> Result<Vec<TendrilReport<ActionLog>>, SetupError> {
        assert_eq!(mode, self.ta_exp_mode);
        assert_eq!(filter, self.ta_exp_filter);
        assert_eq!(dry_run, self.ta_exp_dry_run);
        assert_eq!(force, self.ta_exp_force);
        if let Some(p) = td_repo {
            assert_eq!(p.inner(), self.ta_exp_path.unwrap());
        }
        else {
            assert_eq!(None, self.ta_exp_path);
        }

        if let Some(f) = self.ta_fn.as_ref() {
            f(mode, td_repo, filter, dry_run, force)
        }
        else {
            self.ta_const_rt.clone()
        }
    }
}

/// Abstractions for Tendrils test setups built around temporary directories.
pub struct Setup {
    pub temp_dir: TempDir, // Must return a reference to keep it in scope
    pub parent_dir: PathBuf,
    pub td_repo: PathBuf,
    pub dot_td_dir: PathBuf,
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
        let td_repo = temp_dir.path().join("TendrilsRepo");
        let dot_td_dir = td_repo.join(".tendrils");
        let td_json_file = dot_td_dir.join("tendrils.json");
        let group_dir = td_repo.join("SomeApp");
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
            td_repo,
            dot_td_dir,
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

    pub fn file_tendril_raw(&self) -> RawTendril {
        let raw = RawTendril {
            local: "SomeApp/misc.txt".to_string(),
            remote: self.remote_file.to_string_lossy().to_string(),
            mode: TendrilMode::DirOverwrite,
            profiles: vec![],
        };
        raw
    }

    #[allow(private_interfaces)]
    pub fn file_tendril(&self) -> Tendril {
        Tendril::new_expose(
            self.uni_td_repo(),
            PathBuf::from("SomeApp/misc.txt"),
            self.parent_dir.join("misc.txt").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()
    }

    #[allow(private_interfaces)]
    pub fn dir_tendril(&self) -> Tendril {
        Tendril::new_expose(
            self.uni_td_repo(),
            PathBuf::from("SomeApp/misc"),
            self.parent_dir.join("misc").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()
    }

    #[allow(private_interfaces)]
    pub fn subdir_file_tendril(&self) -> Tendril {
        Tendril::new_expose(
            self.uni_td_repo(),
            PathBuf::from("SomeApp/SubDir/misc.txt"),
            self.parent_dir.join("SubDir/misc.txt").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()
    }

    #[allow(private_interfaces)]
    pub fn subdir_dir_tendril(&self) -> Tendril {
        Tendril::new_expose(
            self.uni_td_repo(),
            PathBuf::from("SomeApp/SubDir/misc"),
            self.parent_dir.join("SubDir/misc").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()
    }

    pub fn make_parent_dir(&self) {
        create_dir_all(&self.parent_dir).unwrap();
    }

    pub fn make_td_repo_dir(&self) {
        create_dir_all(&self.td_repo).unwrap();
    }

    pub fn make_dot_td_dir(&self) {
        create_dir_all(&self.dot_td_dir).unwrap();
    }

    pub fn make_td_json_file(&self, tendrils: &[RawTendril]) {
        self.make_dot_td_dir();
        let json = crate::config::serialize_config(Config {
            raw_tendrils: tendrils.to_vec(),
        });
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

    /// Note: This changes the `HOME` environment variable for the process
    /// so should not be run in parallel with other tests where this may
    /// interfere.
    pub fn make_global_cfg_dir(&self) {
        self.set_home_dir();
        create_dir_all(global_cfg_dir()).unwrap();
    }

    /// Note: This changes the `HOME` environment variable for the process
    /// so should not be run in parallel with other tests where this may
    /// interfere.
    pub fn make_global_cfg_file(&self, json: String) {
        self.make_global_cfg_dir();
        write(global_cfg_file(), json).unwrap();
    }

    /// Sets the home directory to the [`Setup::temp_dir`].
    /// Note: This changes the `HOME` environment variable for the process
    /// so should not be run in parallel with other tests where this may
    /// interfere.
    pub fn set_home_dir(&self) {
        std::env::set_var("HOME", self.temp_dir.path());
    }

    pub fn td_json_file_contents(&self) -> String {
        read_to_string(&self.td_json_file).unwrap()
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

    pub(crate) fn uni_td_repo(&self) -> UniPath {
        UniPath::from(&self.td_repo)
    }
}

