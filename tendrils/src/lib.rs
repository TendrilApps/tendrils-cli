//! Provides tools for managing tendrils.
//! See also the [`td` CLI](..//td/index.html)

mod config;
mod enums;
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
use path_ext::{PathExt, UniPath};
use std::fs::{create_dir_all, remove_dir_all, remove_file};
use std::path::{Path, PathBuf};
mod tendril;
use tendril::Tendril;
pub use tendril::TendrilBundle;
mod tendril_report;
pub use tendril_report::{ActionLog, TendrilLog, TendrilReport};

#[cfg(test)]
mod tests;
#[cfg(test)]
// Must be included in top level of the crate (see rstest_reuse docs)
use rstest_reuse;

#[cfg(any(test, feature = "_test_utils"))]
pub mod test_utils;

/// Represents the public Tendrils API.
/// Although the API functions are not static (i.e. they
/// require an API instance), this is mainly to facilitate easier mocking
/// for testing. The actual API implementation should have little to no state.
pub trait TendrilsApi {
    /// Returns the `default-repo-path` value stored in
    /// `~/.tendrils/global-config.json` or any [errors](GetConfigError) that
    /// occur. Returns `None` if the value is blank or absent, or if the file
    /// does not exist. Note: This does *not* check whether the folder
    /// [is a tendrils repo](`TendrilsApi::is_tendrils_repo`).
    fn get_default_repo_path(&self) -> Result<Option<PathBuf>, GetConfigError>;

    /// Initializes a Tendrils repo with a `.tendrils` folder and a
    /// pre-populated `tendrils.json` file. This will fail if the folder is
    /// already a Tendrils repo or if there are general file-system errors.
    /// This will also fail if the folder is not empty and `force` is false.
    ///
    /// # Arguments
    /// - `dir` - The folder to initialize
    /// - `force` - Ignores the [`InitError::NotEmpty`] error
    fn init_tendrils_repo(&self, dir: &Path, force: bool) -> Result<(), InitError>;

    /// Returns `true` if the given folder is a Tendrils repo, otherwise
    /// `false`.
    /// - A Tendrils repo is defined by having a `.tendrils` subfolder with
    /// a `tendrils.json` file in it.
    /// - Note: This does *not* check that the `tendrils.json` contents are valid.
    fn is_tendrils_repo(&self, dir: &Path) -> bool;

    /// Reads the `tendrils.json` file in the given Tendrils repo, and
    /// performs the action on each tendril that matches the
    /// filter.
    ///
    /// The order of the actions maintains the order of the tendril bundles found in
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
    /// - `update_fn` - Updater function that will be passed the most recent
    /// report as each action is completed. This allows the calling function to
    /// receive updates as progress is made.
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
    /// A given [`TendrilBundle`] may result in many actions if it includes
    /// multiple names and/or parents. Returns a [`SetupError`] if there are
    /// any issues in setting up the batch of actions.
    fn tendril_action_updating<F: FnMut(TendrilReport<ActionLog>)>(
        &self,
        update_fn: F,
        mode: ActionMode,
        td_repo: Option<&Path>,
        filter: FilterSpec,
        dry_run: bool,
        force: bool,
    ) -> Result<(), SetupError>; 

    /// Same behaviour as [`tendril_action_updating`](`TendrilsApi::tendril_action_updating`) except reports are only
    /// returned once all actions have completed.
    fn tendril_action(
        &self,
        mode: ActionMode,
        td_repo: Option<&Path>,
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

    fn init_tendrils_repo(&self, dir: &Path, force: bool) -> Result<(), InitError> {
        init_tendrils_repo(&UniPath::from(dir), force)
    }

    fn is_tendrils_repo(&self, dir: &Path) -> bool {
        is_tendrils_repo(&UniPath::from(dir))
    }

    fn tendril_action_updating<F: FnMut(TendrilReport<ActionLog>)>(
        &self,
        update_fn: F,
        mode: ActionMode,
        td_repo: Option<&Path>,
        filter: FilterSpec,
        dry_run: bool,
        force: bool,
    ) -> Result<(), SetupError> {
        let resolved_td_repo;
        let passed_repo;
        if let Some(v) = td_repo {
            resolved_td_repo = UniPath::from(v);
            passed_repo = Some(&resolved_td_repo);
        }
        else {
            passed_repo = None;
        }
        tendril_action_updating(
            update_fn,
            mode,
            passed_repo,
            filter,
            dry_run,
            force
        )
    }

    fn tendril_action(
        &self,
        mode: ActionMode,
        td_repo: Option<&Path>,
        filter: FilterSpec,
        dry_run: bool,
        force: bool,
    ) -> Result<Vec<TendrilReport<ActionLog>>, SetupError> {
        let mut reports = vec![];
        let updater = |r| reports.push(r);

        self.tendril_action_updating(updater, mode, td_repo, filter, dry_run, force)?;
        Ok(reports)
    }
}

const INIT_TD_TENDRILS_JSON: &str = r#"{
    "tendrils": [
        {
            "group": "SomeApp",
            "names": "SomeFile.ext",
            "parents": "path/to/containing/folder"
        },
        {
            "group": "SomeApp2",
            "names": ["SomeFile2.ext", "SomeFolder3"],
            "parents": [
                "path/to/containing/folder2",
                "path/to/containing/folder3",
                "path/to/containing/folder4"
            ],
            "dir-merge": false,
            "link": true,
            "profiles": ["home", "work"]
        }
    ]
}
"#;

fn init_tendrils_repo(dir: &UniPath, force: bool) -> Result<(), InitError> {
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

fn is_tendrils_repo(dir: &UniPath) -> bool {
    dir.inner().join(".tendrils/tendrils.json").is_file()
}

fn tendril_action_updating<F: FnMut(TendrilReport<ActionLog>)>(
    update_fn: F,
    mode: ActionMode,
    td_repo: Option<&UniPath>,
    filter: FilterSpec,
    dry_run: bool,
    force: bool,
) -> Result<(), SetupError> {
    let td_repo= get_tendrils_repo(td_repo)?;
    let config = config::get_config(&td_repo)?;
    let all_tendrils = config.tendrils;

    let filtered_tendrils = filter_tendrils(all_tendrils, filter);
    if mode == ActionMode::Link && !filtered_tendrils.is_empty() && !can_symlink() {
        return Err(SetupError::CannotSymlink);
    }

    batch_tendril_action(update_fn, mode, &td_repo, filtered_tendrils, dry_run, force);
    Ok(())
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
        Some(FsoType::Dir | FsoType::SymDir) => {
            prepare_dest(to, dir_merge)?;

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
            prepare_dest(to, false)?;

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
    match (source, dest, force) {
        (None, _, _) => Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Source,
        }),
        (Some(FsoType::Dir), Some(FsoType::File), false) => {
            Err(TendrilActionError::TypeMismatch {
                loc: Location::Dest,
                mistype: FsoType::File,
            })
        }
        (Some(FsoType::File), Some(FsoType::Dir), false) => {
            Err(TendrilActionError::TypeMismatch {
                loc: Location::Dest,
                mistype: FsoType::Dir,
            })
        }
        (_, Some(FsoType::SymFile), false) => {
            Err(TendrilActionError::TypeMismatch {
                loc: Location::Dest,
                mistype: FsoType::SymFile,
            })
        }
        (_, Some(FsoType::SymDir), false) => {
            Err(TendrilActionError::TypeMismatch {
                loc: Location::Dest,
                mistype: FsoType::SymDir,
            })
        }
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
        _ => Ok(()),
    }
}

/// Prepares the destination before copying a file system object
/// to it
fn prepare_dest(
    dest: &Path,
    dir_merge: bool,
) -> Result<(), TendrilActionError> {
    if !dir_merge && dest.is_dir() {
        if let Err(e) = remove_dir_all(dest) {
            return Err(TendrilActionError::IoError {
                kind: e.kind(),
                loc: Location::Dest,
            });
        }
    }
    else if dest.is_file() {
        if let Err(e) = remove_file(dest) {
            return Err(TendrilActionError::IoError {
                kind: e.kind(),
                loc: Location::Dest,
            });
        }
    }

    match create_dir_all(dest.parent().unwrap_or(dest)) {
        Err(e) => Err(TendrilActionError::IoError {
            kind: e.kind(),
            loc: Location::Dest,
        }),
        _ => Ok(()),
    }
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
// checking environment variable
fn get_tendrils_repo(
    starting_path: Option<&UniPath>,
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
        None => match config::get_global_config()?.default_repo_path {
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

fn is_recursive_tendril(td_repo: &UniPath, tendril_full_path: &Path) -> bool {
    let repo_inner = td_repo.inner();
    repo_inner == tendril_full_path
        || repo_inner.ancestors().any(|p| p == tendril_full_path)
        || tendril_full_path.ancestors().any(|p| p == repo_inner)
}

fn link_tendril(
    td_repo: &UniPath,
    tendril: &Tendril,
    dry_run: bool,
    mut force: bool,
) -> ActionLog {
    let target = tendril.local_path(td_repo);
    let create_at = tendril.full_path();

    let mut log = ActionLog::new(
        target.get_type(),
        create_at.get_type(),
        create_at,
        Ok(TendrilActionSuccess::New), // Init only value
    );
    if tendril.mode != TendrilMode::Link {
        log.result = Err(TendrilActionError::ModeMismatch);
        return log;
    }
    if is_recursive_tendril(td_repo, log.resolved_path()) {
        log.result = Err(TendrilActionError::Recursion);
        return log;
    }
    if !tendril.parent().exists() {
        log.result = Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Dest,
        });
        return log;
    }

    let local_type;
    if td_repo.inner().exists() && log.local_type().is_none() {
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
    td_repo: &UniPath,
    tendril: &Tendril,
    dry_run: bool,
    force: bool,
) -> ActionLog {
    let dest = tendril.local_path(td_repo);
    let source = tendril.full_path();

    let mut log = ActionLog::new(
        dest.get_type(),
        source.get_type(),
        source,
        Ok(TendrilActionSuccess::New), // Init only value
    );

    if tendril.mode == TendrilMode::Link {
        log.result = Err(TendrilActionError::ModeMismatch);
        return log;
    }
    else if is_recursive_tendril(td_repo, log.resolved_path()) {
        log.result = Err(TendrilActionError::Recursion);
        return log;
    }
    else if !td_repo.inner().exists() {
        log.result = Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Dest,
        });
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
    td_repo: &UniPath,
    tendril: &Tendril,
    dry_run: bool,
    force: bool,
) -> ActionLog {
    let source = tendril.local_path(td_repo);
    let dest = tendril.full_path();

    let mut log = ActionLog::new(
        source.get_type(),
        dest.get_type(),
        dest,
        Ok(TendrilActionSuccess::New), // Init only value
    );
    if tendril.mode == TendrilMode::Link {
        log.result = Err(TendrilActionError::ModeMismatch);
        return log;
    }
    if is_recursive_tendril(td_repo, log.resolved_path()) {
        log.result = Err(TendrilActionError::Recursion);
        return log;
    }
    if !tendril.parent().exists() {
        log.result = Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Dest,
        });
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

fn batch_tendril_action<F: FnMut(TendrilReport<ActionLog>)>(
    mut update_fn: F,
    mode: ActionMode,
    td_repo: &UniPath,
    td_bundles: Vec<TendrilBundle>,
    dry_run: bool,
    force: bool,
) {
    let first_only = mode == ActionMode::Pull;
    let can_symlink =
        (mode == ActionMode::Link || mode == ActionMode::Out) && can_symlink();

    for bundle in td_bundles.into_iter() {
        let bundle_rc = std::rc::Rc::new(bundle);
        let tendrils = bundle_rc.resolve_tendrils(first_only);

        // The number of parents that were considered when
        // resolving the tendril bundle
        let num_parents = match first_only {
            true => 1,
            false => bundle_rc.parents.len(),
        };

        for (i, tendril) in tendrils.into_iter().enumerate() {
            let log = match (tendril, &mode, can_symlink) {
                (Ok(v), ActionMode::Pull, _) => {
                    Ok(pull_tendril(td_repo, &v, dry_run, force))
                }
                (Ok(v), ActionMode::Push, _) => {
                    Ok(push_tendril(td_repo, &v, dry_run, force))
                }
                (Ok(v), ActionMode::Out, _) if v.mode != TendrilMode::Link => {
                    Ok(push_tendril(td_repo, &v, dry_run, force))
                }
                (Ok(v), ActionMode::Out | ActionMode::Link, true) => {
                    Ok(link_tendril(td_repo, &v, dry_run, force))
                }
                (Ok(v), ActionMode::Link | ActionMode::Out, false) => {
                    // Do not attempt to symlink if it has already been
                    // determined that the process
                    // does not have the required permissions.
                    // This prevents deleting any of the remote files
                    // unnecessarily.
                    let remote = v.full_path();
                    Ok(ActionLog::new(
                        v.local_path(td_repo).get_type(),
                        remote.get_type(),
                        remote,
                        Err(TendrilActionError::IoError {
                            kind: std::io::ErrorKind::PermissionDenied,
                            loc: Location::Dest,
                        }),
                    ))
                }
                (Err(e), _, _) => Err(e),
            };

            let name_idx = ((i / num_parents) as f32).floor() as usize;
            let name = bundle_rc.names[name_idx].clone();

            let report = TendrilReport {
                orig_tendril: std::rc::Rc::clone(&bundle_rc),
                name,
                log,
            };

            update_fn(report);
        }
    }
}
