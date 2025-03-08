//! - Provides core functionality for the [`tendrils-cli`](https://crates.io/crates/tendrils-cli) crate and its `td` CLI tool
//! - See documentation at <https://github.com/TendrilApps/tendrils-cli>

mod config;
mod enums;
use config::{get_config, LazyCachedGlobalConfig};
pub use enums::{
    ActionMode,
    ConfigType,
    FsoType,
    GetConfigError,
    GetTendrilsRepoError,
    InitError,
    InvalidTendrilError,
    Location,
    TendrilActionError,
    TendrilActionSuccess,
    SetupError,
    TendrilMode,
};
mod env_ext;
use env_ext::can_symlink;
mod filtering;
use filtering::filter_tendrils;
pub use filtering::FilterSpec;
mod path_ext;
use path_ext::PathExt;
pub use path_ext::UniPath;
use std::fs::{create_dir_all, remove_dir_all, remove_file};
use std::path::{Path, PathBuf};
mod tendril;
use tendril::Tendril;
pub use tendril::RawTendril;
mod tendril_report;
pub use tendril_report::{
    ActionLog,
    CallbackUpdater,
    ListLog,
    TendrilLog,
    TendrilReport,
    UpdateHandler
};

#[cfg(test)]
mod tests;

#[cfg(any(test, feature = "_test_utils"))]
pub mod test_utils;

/// Represents the public Tendrils API.
/// Although the API functions are not static (i.e. they
/// require an API instance), this is mainly to facilitate easier mocking
/// for testing. The actual API implementation should have little to no state.
pub trait TendrilsApi {
    /// Returns the `default-repo-path` value stored in
    /// `~/.tendrils/global-config.json` or any [errors](GetConfigError) that
    /// occur. Returns `None` if the value is blank or absent, or if the config
    /// file does not exist. Note: This does *not* check whether the folder
    /// [is a tendrils repo](`TendrilsApi::is_tendrils_repo`).
    fn get_default_repo_path(&self) -> Result<Option<PathBuf>, GetConfigError>;

    /// Returns the `default-profiles` stored in
    /// `~/.tendrils/global-config.json` or any [errors](GetConfigError) that
    /// occur. Returns `None` if the value is blank or absent, or if the config
    /// file does not exist.
    fn get_default_profiles(&self) -> Result<Option<Vec<String>>, GetConfigError>;

    /// Initializes a Tendrils repo with a `.tendrils` folder and a
    /// pre-populated `tendrils.json` file. This will fail if the folder is
    /// already a Tendrils repo or if there are general file-system errors.
    /// This will also fail if the folder is not empty and `force` is false.
    ///
    /// # Arguments
    /// - `dir` - The folder to initialize
    /// - `force` - Ignores the [`InitError::NotEmpty`] error
    fn init_tendrils_repo(&self, dir: &UniPath, force: bool) -> Result<(), InitError>;

    /// Returns `true` if the given folder is a Tendrils repo, otherwise
    /// `false`.
    /// - A Tendrils repo is defined by having a `.tendrils` subfolder with
    /// a `tendrils.json` file in it.
    /// - Note: This does *not* check that the `tendrils.json` contents are valid.
    fn is_tendrils_repo(&self, dir: &UniPath) -> bool;

    fn list_tendrils(
        &self,
        td_repo: Option<&UniPath>,
        filter: FilterSpec,
    ) -> Result<Vec<TendrilReport<ListLog>>, SetupError>;
    
    /// Reads the `tendrils.json` file in the given Tendrils repo, and
    /// performs the action on each tendril that matches the
    /// filter.
    ///
    /// The order of the actions maintains the order of the [`RawTendril`]s found in
    /// the `tendrils.json`, but each one is expanded into individual tendrils firstly by
    /// each of its `names`, then by each of its `parents`. For example, for a
    /// list of two tendril bundles [t1, t2], each having multiple names [n1, n2] and
    /// multiple parents [p1, p2], the list will be expanded to:
    /// - t1_n1_p1
    /// - t1_n1_p2
    /// - t1_n2_p1
    /// - t1_n2_p2
    /// - t2_n1_p1
    /// - t2_n1_p2
    /// - t2_n2_p1
    /// - t2_n2_p2
    ///
    /// # Arguments
    /// - `updater` - [`UpdateHandler`] to provide synchronous progress updates
    /// to the caller.
    /// - `mode` - The action mode to be performed.
    /// - `td_repo` - The Tendrils repo to perform the actions on. If given
    /// `None`, the [default repo](`TendrilsApi::get_default_repo_path`) will be checked for a
    /// valid Tendrils repo. If neither the given `td_repo` or the default
    /// folder are valid Tendrils folders, a
    /// [`SetupError::NoValidTendrilsRepo`] is returned. 
    /// - `filter` - Only tendrils matching this filter will be included.
    /// - `dry_run`
    ///     - `true` will perform the internal checks for the action but does not
    /// modify anything on the file system. If the action is expected to fail, the
    /// expected [`TendrilActionError`] is returned. If it's expected to succeed,
    /// it returns [`TendrilActionSuccess::NewSkipped`] or
    /// [`TendrilActionSuccess::OverwriteSkipped`]. Note: It is still possible
    /// for a successful dry run to fail in an actual run.
    ///     - `false` will perform the action normally (modifying the file system),
    /// and will return [`TendrilActionSuccess::New`] or
    /// [`TendrilActionSuccess::Overwrite`] if successful.
    /// - `force`
    ///     - `true` will ignore any type mismatches and will force the operation.
    ///     - `false` will simply return [`TendrilActionError::TypeMismatch`] if
    /// there is a type mismatch.
    ///
    /// # Returns
    /// A [`TendrilReport`] containing an [`ActionLog`] for each tendril action.
    /// Returns a [`SetupError`] if there are any issues in setting up the
    /// batch of actions.
    fn tendril_action_updating<U> (
        &self,
        updater: U,
        mode: ActionMode,
        td_repo: Option<&UniPath>,
        filter: FilterSpec,
        dry_run: bool,
        force: bool,
    )
    -> Result<(), SetupError>
    where
        U: UpdateHandler<ActionLog>;

    /// Same behaviour as [`tendril_action_updating`](`TendrilsApi::tendril_action_updating`) except reports are only
    /// returned once all actions have completed.
    fn tendril_action(
        &self,
        mode: ActionMode,
        td_repo: Option<&UniPath>,
        filter: FilterSpec,
        dry_run: bool,
        force: bool,
    ) -> Result<Vec<TendrilReport<ActionLog>>, SetupError>;
}

pub struct TendrilsActor {}

impl TendrilsApi for TendrilsActor {
    fn get_default_repo_path(&self) -> Result<Option<PathBuf>, GetConfigError> {
        Ok(config::get_global_config()?.default_repo_path)
    }

    fn get_default_profiles(&self) -> Result<Option<Vec<String>>, GetConfigError> {
        Ok(config::get_global_config()?.default_profiles)
    }

    fn init_tendrils_repo(&self, dir: &UniPath, force: bool) -> Result<(), InitError> {
        if !dir.inner().exists() {
            return Err(InitError::IoError { kind: std::io::ErrorKind::NotFound });
        }
        else if is_tendrils_repo(dir) {
            return Err(InitError::AlreadyInitialized);
        }
        else if !force && std::fs::read_dir(dir.inner())?.count() > 0 {
            return Err(InitError::NotEmpty);
        }

        let td_dot_json_dir = dir.inner().join(".tendrils");
        let td_json_file = td_dot_json_dir.join("tendrils.json");
        if !td_dot_json_dir.exists() {
            std::fs::create_dir(td_dot_json_dir)?;
        }
        Ok(std::fs::write(td_json_file, INIT_TD_TENDRILS_JSON)?)
    }

    fn is_tendrils_repo(&self, dir: &UniPath) -> bool {
        is_tendrils_repo(dir)
    }

    fn list_tendrils(
        &self,
        td_repo: Option<&UniPath>,
        filter: FilterSpec,
    ) -> Result<Vec<TendrilReport<ListLog>>, SetupError> {
        let mut global_cfg = LazyCachedGlobalConfig::new();
        let td_repo= get_tendrils_repo(td_repo, &mut global_cfg)?;
        let all_tendrils = get_config(&td_repo)?.raw_tendrils;
        let filtered_tendrils = 
            filter_tendrils(all_tendrils, filter, &mut global_cfg);
        let mut reports = Vec::with_capacity(filtered_tendrils.len());

        for raw_tendril in filtered_tendrils {
            let log = match raw_tendril.resolve(&td_repo) {
                Ok(v) => {
                    Ok(ListLog::new(
                        v.local_abs().get_type(), 
                        v.remote().inner().get_type(),
                        v.remote().inner().into()
                    ))
                }
                Err(e) => Err(e),
            };

            reports.push(TendrilReport {
                raw_tendril,
                log,
            });
        }

        Ok(reports)
    }


    fn tendril_action_updating<U>(
        &self,
        updater: U,
        mode: ActionMode,
        td_repo: Option<&UniPath>,
        filter: FilterSpec,
        dry_run: bool,
        force: bool,
    ) -> Result<(), SetupError>
    where
        U: UpdateHandler<ActionLog>,
    {
        let mut global_cfg = LazyCachedGlobalConfig::new();
        let td_repo= get_tendrils_repo(td_repo, &mut global_cfg)?;
        let config = config::get_config(&td_repo)?;
        let all_tendrils = config.raw_tendrils;

        let filtered_tendrils =
            filter_tendrils(all_tendrils, filter, &mut global_cfg);
        if mode == ActionMode::Link && !filtered_tendrils.is_empty() && !can_symlink() {
            return Err(SetupError::CannotSymlink);
        }

        batch_tendril_action(updater, mode, &td_repo, filtered_tendrils, dry_run, force);
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
        let mut reports = vec![];
        let count_fn = |_| {};
        let before_action_fn = |_| {};
        let after_action_fn = |r| reports.push(r);
        let updater = CallbackUpdater::<_, _, _, ActionLog>::new(
            count_fn,
            before_action_fn,
            after_action_fn,
        );

        self.tendril_action_updating(updater, mode, td_repo, filter, dry_run, force)?;
        Ok(reports)
    }
}

const INIT_TD_TENDRILS_JSON: &str = r#"{
    "tendrils": {
        "SomeApp/SomeFile.ext": {
            "remotes": "/path/to/SomeFile.ext"
        },
        "SomeApp2/SomeFolder": {
            "remotes": [
                "/path/to/SomeFolder",
                "/path/to/DifferentName",
                "~/path/in/home/dir/SomeFolder",
                "/path/using/<MY-ENV-VAR>/SomeFolder"
            ],
            "dir-merge": false,
            "link": true,
            "profiles": ["home", "work"]
        },
        "SomeApp3/file.txt": [
            {
                "remotes": "~/unix/specific/path/file.txt",
                "link": true,
                "profiles": "unix"
            },
            {
                "remotes": [
                    "~/windows/specific/path/file.txt",
                    "~/windows/another-specific/path/file.txt"
                ],
                "link": false,
                "profiles": "windows"
            }
        ]
    }
}
"#;

fn is_tendrils_repo(dir: &UniPath) -> bool {
    dir.inner().join(".tendrils/tendrils.json").is_file()
}

fn copy_fso(
    from: &Path,
    from_type: &Option<FsoType>,
    mut to: &Path,
    to_type: &Option<FsoType>,
    dir_merge: bool,
    dry_run: bool,
    force: bool,
) -> Result<TendrilActionSuccess, TendrilActionError> {
    use std::io::ErrorKind::{NotFound, PermissionDenied};
    let to_existed = to_type.is_some();

    check_copy_types(from_type, to_type, force)?;

    match (dry_run, to_existed) {
        (true, true) => return Ok(TendrilActionSuccess::OverwriteSkipped),
        (true, false) => return Ok(TendrilActionSuccess::NewSkipped),
        _ => {}
    }
    match from_type {
        Some(FsoType::Dir | FsoType::SymDir | FsoType::BrokenSym) => {
            prepare_dest(to, to_type, dir_merge)?;

            to = to.parent().unwrap_or(to);

            let mut copy_opts = fs_extra::dir::CopyOptions::new();
            copy_opts.overwrite = true;
            copy_opts.skip_exist = false;
            match (fs_extra::dir::copy(from, to, &copy_opts), to_existed) {
                (Ok(_v), true) => Ok(TendrilActionSuccess::Overwrite),
                (Ok(_v), false) => Ok(TendrilActionSuccess::New),
                (Err(e), _) => match e.kind {
                    // Convert fs_extra::errors
                    fs_extra::error::ErrorKind::Io(e) => {
                        if is_rofs_err(&e.kind()) {
                            Err(TendrilActionError::IoError {
                                kind: e.kind(),
                                loc: Location::Dest,
                            })
                        }
                        else {
                            Err(TendrilActionError::from(e))
                        }
                    }
                    fs_extra::error::ErrorKind::PermissionDenied => {
                        let loc = which_copy_perm_failed(to);
                        Err(TendrilActionError::IoError {
                            kind: PermissionDenied,
                            loc,
                        })
                    }
                    _ => {
                        Err(TendrilActionError::from(std::io::ErrorKind::Other))
                    }
                },
            }
        }
        Some(FsoType::File | FsoType::SymFile) => {
            prepare_dest(to, to_type, false)?;

            match (std::fs::copy(from, to), to_existed) {
                (Ok(_v), true) => Ok(TendrilActionSuccess::Overwrite),
                (Ok(_v), false) => Ok(TendrilActionSuccess::New),
                (Err(e), _) if e.kind() == PermissionDenied => {
                    let loc = which_copy_perm_failed(to);
                    Err(TendrilActionError::IoError {
                        kind: PermissionDenied,
                        loc,
                    })
                }
                (Err(e), _) if is_rofs_err(&e.kind()) => {
                    Err(TendrilActionError::IoError {
                        kind: e.kind(),
                        loc: Location::Dest,
                    })
                }
                (Err(e), _) => Err(TendrilActionError::from(e)),
            }
        }
        None => Err(TendrilActionError::IoError {
            kind: NotFound,
            loc: Location::Source,
        }),
    }
}

/// Returns [`Err(TendrilActionError::TypeMismatch)`](TendrilActionError::TypeMismatch)
/// if the type (file vs folder) of the source and destination are mismatched,
/// or if either the source or destination are symlinks. If `force` is true,
/// type mismatches are ignored.
/// Returns an [`Err(TendrilActionError::IoError)`](TendrilActionError::IoError)
/// if the `source` does not exist.
/// Otherwise, returns `Ok(())`.
///
/// No other invariants of [`TendrilActionError`] are returned.
///
/// Note: This is not applicable in link mode - see [`check_symlink_types`]
/// instead.
fn check_copy_types(
    source: &Option<FsoType>,
    dest: &Option<FsoType>,
    force: bool,
) -> Result<(), TendrilActionError> {
    match (source, dest) {
        (None | Some(FsoType::BrokenSym), _) => Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Source,
        }),
        (_, _) if force => Ok(()),
        (Some(s), _) if s.is_symlink() => Err(TendrilActionError::TypeMismatch {
            loc: Location::Source,
            mistype: s.to_owned(),
        }),
        (Some(s), Some(d)) if s != d => Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: d.to_owned(),
        }),
        (Some(_), _) => Ok(()),
    }
}

/// Prepares the destination before copying a file system object
/// to it
fn prepare_dest(
    dest: &Path,
    dest_type: &Option<FsoType>,
    dir_merge: bool,
) -> Result<(), TendrilActionError> {
    match (dest_type, dir_merge) {
        (Some(d), false) if d.is_dir() => {
            if let Err(e) = remove_dir_all(dest) {
                return Err(TendrilActionError::IoError {
                    kind: e.kind(),
                    loc: Location::Dest,
                });
            }
        }
        (Some(d), _) if d.is_file() => {
            if let Err(e) = remove_file(dest) {
                return Err(TendrilActionError::IoError {
                    kind: e.kind(),
                    loc: Location::Dest,
                });
            }
        }
        (Some(FsoType::BrokenSym), _) => remove_symlink(&dest)?,
        (_, _) => {},
    };

    match create_dir_all(dest.parent().unwrap_or(dest)) {
        Err(e) => Err(TendrilActionError::IoError {
            kind: e.kind(),
            loc: Location::Dest,
        }),
        _ => Ok(()),
    }
}

fn remove_symlink(path: &Path) -> Result<(), std::io::Error> {
    // Since there's no easy way to determine the type of a broken symlink,
    // just try deleting it as a file then fall back to deleting it as a
    // directory.
    // Another potential option:
    // https://gitlab.com/chris-morgan/symlink/-/blob/master/src/windows/mod.rs?ref_type=heads
    #[cfg(windows)]
    if remove_file(path).is_err() {
        remove_dir_all(path)
    }
    else {
        Ok(())
    }

    #[cfg(not(windows))]
    remove_file(&path)
}

fn which_copy_perm_failed(to: &Path) -> Location {
    match to.parent() {
        Some(p) if p.parent().is_none() => Location::Dest, // Is root
        Some(p) => match p.metadata() {
            Ok(md) if md.permissions().readonly() => Location::Dest,
            Ok(_) => Location::Source,
            _ => Location::Unknown,
        },
        None => Location::Dest,
    }
}

fn is_rofs_err(e_kind: &std::io::ErrorKind) -> bool {
    // Possible bug where the std::io::ErrorKind::ReadOnlyFilesystem
    // is only available in nightly but is being returned on Mac
    format!("{:?}", e_kind).contains("ReadOnlyFilesystem")
}

/// Looks for a Tendrils repo (as defined by [`TendrilsApi::is_tendrils_repo`])
/// - If given a `starting_path`, it begins looking in that folder.
///     - If it is a Tendrils repo, `starting_path` is returned
///     - Otherwise [`GetTendrilsRepoError::GivenInvalid`] is returned.
/// - If a `starting_path` is not provided, the
/// [default repo](`TendrilsApi::get_default_repo_path`) is used.
///     - If it points to a valid repo, that path is returned
///     - If it points to an invalid folder,
/// [`GetTendrilsRepoError::DefaultInvalid`] is returned
///     - If it is not set,
/// [`GetTendrilsRepoError::DefaultNotSet`] is returned.
// TODO: Recursively look through all parent folders before
// checking global config?
fn get_tendrils_repo(
    starting_path: Option<&UniPath>,
    global_cfg: &mut LazyCachedGlobalConfig,
) -> Result<UniPath, GetTendrilsRepoError> {
    match starting_path {
        Some(v) => {
            if is_tendrils_repo(&v) {
                Ok(v.to_owned())
            }
            else {
                Err(GetTendrilsRepoError::GivenInvalid {
                    path: PathBuf::from(v.inner()),
                })
            }
        }
        None => match global_cfg.eval()?.default_repo_path {
            Some(v) => {
                let u_path = UniPath::from(v);
                if is_tendrils_repo(&u_path) {
                    Ok(u_path)
                }
                else {
                    Err(GetTendrilsRepoError::DefaultInvalid {
                        path: PathBuf::from(u_path.inner()),
                    })
                }
            }
            None => Err(GetTendrilsRepoError::DefaultNotSet),
        }
    }
}

fn link_tendril(
    tendril: &Tendril,
    dry_run: bool,
    mut force: bool,
) -> ActionLog {
    let target = tendril.local_abs();
    let create_at = tendril.remote().inner();

    let mut log = ActionLog::new(
        target.get_type(),
        create_at.get_type(),
        create_at.to_path_buf(),
        Ok(TendrilActionSuccess::New), // Init only value
    );
    if tendril.mode != TendrilMode::Link {
        log.result = Err(TendrilActionError::ModeMismatch);
        return log;
    }

    let local_type;
    if log.local_type().is_none()
        || log.local_type() == &Some(FsoType::BrokenSym) {
        if log.local_type() == &Some(FsoType::BrokenSym) {
            if force {
                if !dry_run {
                    if let Err(e) = remove_symlink(&target) {
                        log.result = Err(e.into());
                        return log;
                    }
                }
            }
            else {
                log.result = Err(TendrilActionError::TypeMismatch {
                    mistype: FsoType::BrokenSym,
                    loc: Location::Source
                });
                return log;
            }
        }

        // Local does not exist - copy it first
        if let Err(e) = copy_fso(
            log.resolved_path(),
            log.remote_type(),
            &target,
            &None,
            false,
            dry_run,
            false,
        ) {
            log.result = Err(e);
            return log;
        };
        local_type = log.remote_type();
        force = true;
    }
    else {
        local_type = log.local_type();
    }

    log.result = symlink(
        log.resolved_path(),
        log.remote_type(),
        &target,
        local_type,
        dry_run,
        force,
    );

    log
}

fn pull_tendril(
    tendril: &Tendril,
    dry_run: bool,
    force: bool,
) -> ActionLog {
    let dest = tendril.local_abs();
    let source = tendril.remote().inner();

    let mut log = ActionLog::new(
        dest.get_type(),
        source.get_type(),
        source.to_path_buf(),
        Ok(TendrilActionSuccess::New), // Init only value
    );

    if tendril.mode == TendrilMode::Link {
        log.result = Err(TendrilActionError::ModeMismatch);
        return log;
    }

    let dir_merge = tendril.mode == TendrilMode::DirMerge;
    log.result = copy_fso(
        log.resolved_path(),
        log.remote_type(),
        &dest,
        log.local_type(),
        dir_merge,
        dry_run,
        force,
    );

    log
}

fn push_tendril(
    tendril: &Tendril,
    dry_run: bool,
    force: bool,
) -> ActionLog {
    let source = tendril.local_abs();
    let dest = tendril.remote().inner();

    let mut log = ActionLog::new(
        source.get_type(),
        dest.get_type(),
        dest.to_path_buf(),
        Ok(TendrilActionSuccess::New), // Init only value
    );
    if tendril.mode == TendrilMode::Link {
        log.result = Err(TendrilActionError::ModeMismatch);
        return log;
    }

    let dir_merge = tendril.mode == TendrilMode::DirMerge;
    log.result = copy_fso(
        &source,
        log.local_type(),
        log.resolved_path(),
        log.remote_type(),
        dir_merge,
        dry_run,
        force,
    );

    log
}

/// Returns [`Err(TendrilActionError::TypeMismatch)`](TendrilActionError::TypeMismatch)
/// if the type of the source and destination are mismatched. If `force` is
/// true, type mismatches are ignored.
/// Returns an [`Err(TendrilActionError::IoError)`](TendrilActionError::IoError)
/// if the `target` does not exist.
/// Otherwise, returns `Ok(())`.
///
/// No other invariants of [`TendrilActionError`] are returned.
///
/// Note: This is not applicable in copy mode - see [`check_copy_types`]
/// instead.
fn check_symlink_types(
    target: &Option<FsoType>,
    create_at: &Option<FsoType>,
    force: bool,
) -> Result<(), TendrilActionError> {
    match (target, create_at, force) {
        (None, _, _) => Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Source,
        }),
        (Some(FsoType::SymFile), _, false) => {
            Err(TendrilActionError::TypeMismatch {
                loc: Location::Source,
                mistype: FsoType::SymFile,
            })
        }
        (Some(FsoType::SymDir), _, false) => {
            Err(TendrilActionError::TypeMismatch {
                loc: Location::Source,
                mistype: FsoType::SymDir,
            })
        }
        (_, Some(FsoType::File), false) => {
            Err(TendrilActionError::TypeMismatch {
                loc: Location::Dest,
                mistype: FsoType::File,
            })
        }
        (_, Some(FsoType::Dir), false) => {
            Err(TendrilActionError::TypeMismatch {
                loc: Location::Dest,
                mistype: FsoType::Dir,
            })
        }
        _ => Ok(()),
    }
}

fn symlink(
    create_at: &Path,
    create_at_type: &Option<FsoType>,
    target: &Path,
    target_type: &Option<FsoType>,
    dry_run: bool,
    force: bool,
) -> Result<TendrilActionSuccess, TendrilActionError> {
    check_symlink_types(target_type, create_at_type, force)?;

    let del_result = match (dry_run, &create_at_type) {
        (true, Some(_)) => return Ok(TendrilActionSuccess::OverwriteSkipped),
        (true, None) => return Ok(TendrilActionSuccess::NewSkipped),
        (false, Some(FsoType::File | FsoType::SymFile)) => {
            remove_file(create_at)
        }
        (false, Some(FsoType::BrokenSym)) => {
            remove_symlink(create_at)
        }
        (false, Some(FsoType::Dir | FsoType::SymDir)) => {
            remove_dir_all(create_at)
        }
        (false, None) => Ok(()),
    };
    match del_result {
        Err(e) => Err(TendrilActionError::IoError {
            kind: e.kind(),
            loc: Location::Dest,
        }),
        _ => Ok(()),
    }?;

    if let Err(e) = create_dir_all(create_at.parent().unwrap_or(create_at)) {
        return Err(TendrilActionError::IoError {
            kind: e.kind(),
            loc: Location::Dest,
        });
    };

    #[cfg(windows)]
    let sym_result = symlink_win(create_at, target);
    #[cfg(unix)]
    let sym_result = symlink_unix(create_at, target);
    match sym_result {
        Err(TendrilActionError::IoError { kind: k, loc: _ }) => {
            Err(TendrilActionError::IoError { kind: k, loc: Location::Dest })
        }
        _ => Ok(()),
    }?;

    if create_at_type.is_none() {
        Ok(TendrilActionSuccess::New)
    }
    else {
        Ok(TendrilActionSuccess::Overwrite)
    }
}

#[cfg(unix)]
fn symlink_unix(
    create_at: &Path,
    target: &Path,
) -> Result<(), TendrilActionError> {
    std::os::unix::fs::symlink(target, create_at)?;

    Ok(())
}

#[cfg(windows)]
fn symlink_win(
    create_at: &Path,
    target: &Path,
) -> Result<(), TendrilActionError> {
    use std::os::windows::fs::{symlink_dir, symlink_file};

    if target.is_dir() {
        symlink_dir(target, create_at)?;
    }
    else {
        symlink_file(target, create_at)?;
    }

    Ok(())
}

fn batch_tendril_action<U>(
    mut updater: U,
    mode: ActionMode,
    td_repo: &UniPath,
    raw_tendrils: Vec<RawTendril>,
    dry_run: bool,
    force: bool,
)
where
    U: UpdateHandler<ActionLog>,
{
    let can_symlink =
        (mode == ActionMode::Link || mode == ActionMode::Out) && can_symlink();
    
    updater.count(raw_tendrils.len() as i32);

    for raw_tendril in raw_tendrils.into_iter() {
        updater.before(raw_tendril.clone());
        let tendril = raw_tendril.resolve(td_repo);

        let log = match (tendril, &mode, can_symlink) {
            (Ok(v), ActionMode::Pull, _) => {
                Ok(pull_tendril(&v, dry_run, force))
            }
            (Ok(v), ActionMode::Push, _) => {
                Ok(push_tendril(&v, dry_run, force))
            }
            (Ok(v), ActionMode::Out, _) if v.mode != TendrilMode::Link => {
                Ok(push_tendril(&v, dry_run, force))
            }
            (Ok(v), ActionMode::Out | ActionMode::Link, true) => {
                Ok(link_tendril(&v, dry_run, force))
            }
            (Ok(v), ActionMode::Link | ActionMode::Out, false) => {
                // Do not attempt to symlink if it has already been
                // determined that the process
                // does not have the required permissions.
                // This prevents deleting any of the remote files
                // unnecessarily.
                let remote = v.remote();
                Ok(ActionLog::new(
                    v.local_abs().get_type(),
                    remote.inner().get_type(),
                    remote.inner().to_path_buf(),
                    Err(TendrilActionError::IoError {
                        kind: std::io::ErrorKind::PermissionDenied,
                        loc: Location::Dest,
                    }),
                ))
            }
            (Err(e), _, _) => Err(e),
        };

        let report = TendrilReport {
            raw_tendril,
            log,
        };

        updater.after(report);
    }
}
