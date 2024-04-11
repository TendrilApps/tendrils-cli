//! Provides tools for managing tendrils.
//! See also the [`td` CLI](..//td/index.html)

mod enums;
pub use enums::{
    ActionMode,
    FsoType,
    GetTendrilsError,
    InitError,
    InvalidTendrilError,
    Location,
    TendrilActionError,
    TendrilActionSuccess,
    TendrilMode,
};
mod filtering;
pub use filtering::{filter_tendrils, FilterSpec};
mod tendril;
use tendril::Tendril;
use std::ffi::OsString;
use std::fs::{create_dir_all, remove_dir_all, remove_file};
use std::path::{Path, PathBuf};
mod tendril_bundle;
pub use tendril_bundle::TendrilBundle;
mod tendril_action_report;
pub use tendril_action_report::TendrilActionReport;

#[cfg(test)]
mod tests;
#[cfg(test)]
// Must be included in top level of the crate (see rstest_reuse docs)
use rstest_reuse;

#[cfg(any(test, feature = "_test_utils"))]
pub mod test_utils;

fn copy_fso(
    from: &Path,
    mut to: &Path,
    dir_merge: bool,
    dry_run: bool,
) -> Result<TendrilActionSuccess, TendrilActionError> {
    use std::io::ErrorKind::{NotFound, PermissionDenied};
    let to_existed = to.exists();

    if from.is_dir() {
        match (dry_run, to_existed) {
            (true, true) => return Ok(TendrilActionSuccess::OverwriteSkipped),
            (true, false) => return Ok(TendrilActionSuccess::NewSkipped),
            _ => {}
        }

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
                    Err(TendrilActionError::from(e))
                },
                fs_extra::error::ErrorKind::PermissionDenied => {
                    let loc = which_copy_perm_failed(&to);
                    Err(TendrilActionError::IoError { kind: PermissionDenied, loc })
                },
                _ => Err(TendrilActionError::from(std::io::ErrorKind::Other))
            }
        }
    }
    else if from.is_file() {
        match (dry_run, to_existed) {
            (true, true) => return Ok(TendrilActionSuccess::OverwriteSkipped),
            (true, false) => return Ok(TendrilActionSuccess::NewSkipped),
            _ => {}
        }

        prepare_dest(to, false)?;

        match (std::fs::copy(from, to), to_existed) {
            (Ok(_v), true) => Ok(TendrilActionSuccess::Overwrite),
            (Ok(_v), false) => Ok(TendrilActionSuccess::New),
            (Err(e), _) if e.kind() == PermissionDenied => {
                let loc = which_copy_perm_failed(&to);
                Err(TendrilActionError::IoError {kind: PermissionDenied, loc})
            },
            (Err(e), _) => Err(TendrilActionError::from(e)),
        }
    }
    else {
        Err(TendrilActionError::IoError {
            kind: NotFound,
            loc: Location::Source
        })
    }
}

/// Returns [`Err(TendrilActionError::TypeMismatch)`](TendrilActionError::TypeMismatch)
/// if the type (file vs folder) of the source and destination are mismatched,
/// otherwise returns `Ok(())`.
/// No other invariants of [`TendrilActionError`] are returned.
/// - Returns error if the source or destination do not exist.
/// - Returns error if either the source or destination are symlinks.
///
/// Note: This is not applicable in link mode.
fn check_copy_types(
    source: &Path, dest: &Path
) -> Result<(), TendrilActionError> {
    if source.is_dir() && dest.is_file() {
        Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: FsoType::File,
        })
    }
    else if source.is_file() && dest.is_dir() {
        Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: FsoType::Dir,
        })
    }
    else if dest.is_symlink() {
        Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: FsoType::Symlink,
        })
    }
    else if source.is_symlink() {
        Err(TendrilActionError::TypeMismatch {
            loc: Location::Source,
            mistype: FsoType::Symlink,
        })
    }
    else {
        Ok(())
    }
}

/// Prepares the destination before copying a file system object
/// to it
fn prepare_dest(dest: &Path, dir_merge: bool) -> Result<(), TendrilActionError> {
    if !dir_merge && dest.is_dir() {
        match remove_dir_all(&dest) {
            Err(e) => return Err(TendrilActionError::IoError {
                kind: e.kind(),
                loc: Location::Dest, 
            }),
            _ => {}
        }
    }
    else if dest.is_file() {
        match remove_file(&dest) {
            Err(e) => return Err(TendrilActionError::IoError {
                kind: e.kind(),
                loc: Location::Dest,
            }),
            _ => {}
        }
    }

    match create_dir_all(dest.parent().unwrap_or(dest)) {
        Err(e) => Err(TendrilActionError::IoError {
            kind: e.kind(),
            loc: Location::Dest,
        }),
        _ => Ok(())
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

/// Parses the `tendrils.json` file in the given *Tendrils* folder
/// and returns the tendril bundles in the order they are defined in the file.
///
/// # Arguments
/// - `td_dir` - Path to the *Tendrils* folder.
pub fn get_tendrils(
    td_dir: &Path,
) -> Result<Vec<TendrilBundle>, GetTendrilsError> {
    let tendrils_file_path = Path::new(&td_dir).join("tendrils.json");
    let tendrils_file_contents = std::fs::read_to_string(tendrils_file_path)?;
    let tendrils = parse_tendrils(&tendrils_file_contents)?;
    Ok(tendrils)
}

/// Looks for a *Tendrils* folder (as defined by [`is_tendrils_dir`])
/// - Begins looking at the `starting_path`. If it is a *Tendrils*
/// folder, the given path is returned.
/// - If it is not a *Tendrils* folder, the environment variable `TENDRILS_DIR`
/// is used. If this variable does not exist or does not point to a
/// valid *Tendrils* folder, then `None` is returned.
// TODO: Recursively look through all parent folders before
// checking environment variable
pub fn get_tendrils_dir(starting_path: &Path) -> Option<PathBuf> {
    if is_tendrils_dir(starting_path) {
        Some(starting_path.to_owned())
    }
    else {
        match std::env::var("TENDRILS_FOLDER") {
            Ok(v) => {
                let test_path = PathBuf::from(v);
                if is_tendrils_dir(&test_path) {
                    Some(test_path)
                }
                else {
                    None
                }
            },
            _ => None
        }
    }
}

const INIT_TD_TENDRILS_JSON: &str =
r#"[
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
"#;

/// Initializes a *Tendrils* folder with a pre-populated `tendrils.json` file.
/// This will fail if the folder is already a *Tendrils* folder or if there are
/// general file-system errors. This will also fail if the folder is not empty and
/// `force` is false.
/// 
/// # Arguments
/// - `dir` - The folder to initialize
/// - `force` - Ignores the [`InitError::NotEmpty`] error
pub fn init_tendrils_dir(dir: &Path, force: bool) -> Result<(), InitError> {
    if !dir.exists() {
        return Err(InitError::IoError {kind: std::io::ErrorKind::NotFound});
    }
    else if is_tendrils_dir(dir) {
        return Err(InitError::AlreadyInitialized);
    }
    else if !force && std::fs::read_dir(dir)?.into_iter().count() > 0 {
        return Err(InitError::NotEmpty);
    }

    let td_json_file = dir.join("tendrils.json");
    Ok(std::fs::write(td_json_file, INIT_TD_TENDRILS_JSON)?)
}

/// Returns `true` if the given folder is a *Tendrils* folder, otherwise `false`.
/// - A *Tendrils* folder is defined by having a `tendrils.json` file in its top level.
/// - Note: This does *not* check that the `tendrils.json` contents are valid.
pub fn is_tendrils_dir(dir: &Path) -> bool {
    dir.join("tendrils.json").is_file()
}

fn is_recursive_tendril(
    td_dir: &Path,
    tendril_full_path: &Path,
) -> bool {
    td_dir == tendril_full_path
        || td_dir.ancestors().any(|p| p == tendril_full_path)
        || tendril_full_path.ancestors().any(|p| p == td_dir)
}

fn link_tendril(
    td_dir: &Path,
    tendril: &Tendril,
    dry_run: bool,
    force: bool,
) -> Result<TendrilActionSuccess, TendrilActionError> {
    let dest= tendril.full_path();
    if tendril.mode != TendrilMode::Link {
        return Err(TendrilActionError::ModeMismatch);
    }
    if is_recursive_tendril(td_dir, &dest) {
        return Err(TendrilActionError::Recursion);
    }
    if !tendril.parent().exists() {
        return Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Dest
        });
    }

    let target = td_dir.join(tendril.group()).join(tendril.name());
    if td_dir.exists() && !target.exists() {
        // Local does not exist - copy it first
        copy_fso(&dest, &target, false, dry_run)?;
    }
    else if !force && !dest.is_symlink() {
        if dest.is_file() {
            return Err(TendrilActionError::TypeMismatch {
                loc: Location::Dest,
                mistype: FsoType::File,
            });
        }
        else if dest.is_dir() {
            return Err(TendrilActionError::TypeMismatch {
                loc: Location::Dest,
                mistype: FsoType::Dir,
            });
        }
    }

    match symlink(&dest, &target, dry_run, force) {
        Err(TendrilActionError::IoError {kind: e_kind, loc: _}) if dry_run
            && e_kind == std::io::ErrorKind::NotFound
            && dest.exists()
            && td_dir.exists() =>
        {
            // Local does not exist and should be copied before link
            // in a non-dry run. Ignore this error here
            Ok(TendrilActionSuccess::OverwriteSkipped)
        }
        result => result,
    }
}

/// # Arguments
/// - `json` - JSON array of tendril bundles
fn parse_tendrils(json: &str) -> Result<Vec<TendrilBundle>, serde_json::Error> {
    serde_json::from_str::<Vec<TendrilBundle>>(json)
}

fn pull_tendril(
    td_dir: &Path,
    tendril: &Tendril,
    dry_run: bool,
    force: bool,
) -> Result<TendrilActionSuccess, TendrilActionError> {
    let source= tendril.full_path();
    if tendril.mode == TendrilMode::Link {
        return Err(TendrilActionError::ModeMismatch);
    }
    else if is_recursive_tendril(td_dir, &source) {
        return Err(TendrilActionError::Recursion);
    }
    else if !td_dir.exists() {
        return Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Dest
        });
    }

    let dest = td_dir.join(tendril.group()).join(tendril.name());

    if !force {
        check_copy_types(&source, &dest)?;
    }

    let dir_merge = tendril.mode == TendrilMode::DirMerge;
    copy_fso(&source, &dest, dir_merge, dry_run)
}

fn push_tendril(
    td_dir: &Path,
    tendril: &Tendril,
    dry_run: bool,
    force: bool,
) -> Result<TendrilActionSuccess, TendrilActionError> {
    let dest= tendril.full_path();
    if tendril.mode == TendrilMode::Link {
        return Err(TendrilActionError::ModeMismatch);
    }
    if is_recursive_tendril(td_dir, &dest) {
        return Err(TendrilActionError::Recursion);
    }
    if !tendril.parent().exists() {
        return Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Dest,
        });
    }

    let source = td_dir.join(tendril.group()).join(tendril.name());

    if !force {
        check_copy_types(&source, &dest)?;
    }

    let dir_merge = tendril.mode == TendrilMode::DirMerge;
    copy_fso(&source, &dest, dir_merge, dry_run)
}

/// Replaces all environment variables in the format `<varname>` in the
/// given path with their values. If the variable is not found, the
/// `<varname>` is left as-is in the path.
/// 
/// The common tilde (`~`) symbol can also be used as a prefix to the path
/// and corresponds to the `HOME` environment variable on Unix/Windows.
/// If `HOME` doesn't exist, it will fallback to a combination of `HOMEDRIVE`
/// and `HOMEPATH` provided they both exist (otherwise the `~` is left as is).
/// This fallback is mainly a Windows specific issue, but is supported on all
/// platforms either way.
/// 
/// Any non UTF-8 characters in a variable's value or in the tilde value
/// are replaced with the U+FFFD replacement character.
/// 
/// # Limitations
/// If the path contains the `<pattern>` and the pattern corresponds to
/// an environment variable, there is no way to escape the brackets
/// to force it to use the raw path. This should only be an issue
/// on Unix (as Windows doesn't allow `<` or `>` in paths anyways),
/// and only when the variable exists (otherwise it uses the raw
/// path). In the future, an escape character such as `|` could be
/// implemented, but this added complexity was avoided for now.
fn resolve_path_variables(mut path: String) -> PathBuf {
    let path_temp = path.clone();
    let vars = parse_env_variables(&path_temp);

    for var in vars {
        let var_no_brkts = &var[1..var.len()-1];
        let os_value = std::env::var_os(var_no_brkts).unwrap_or(OsString::from(var));
        let value = os_value.to_string_lossy();
        path = path.replace(var, &value);
    }

    if path.starts_with('~') {
        path = resolve_tilde(&path);
    }

    PathBuf::from(path)
}

/// Replaces the first instance of `~` with the `HOME` variable
/// and returns the replaced string. If `HOME` doesn't exist,
/// `HOMEDRIVE` and `HOMEPATH` will be combined provided they both exist,
/// otherwise it returns the given string.
/// 
/// Note: This does *not* check that the tilde is the leading character (it could be
/// anywhere in the string) - this check should be done prior to calling this.
fn resolve_tilde(path: &str) -> String {
    use std::env::var_os;
    match var_os("HOME") {
        Some(v) => {
            return path.replacen('~', &v.to_string_lossy(), 1);
        },
        None => ()
    };
    match (var_os("HOMEDRIVE"), var_os("HOMEPATH")) {
        (Some(hd), Some(hp)) => {
            let mut combo = String::from(hd.to_string_lossy());
            combo.push_str(hp.to_string_lossy().as_ref());
            path.replacen('~', &combo, 1)
        },
        _ => String::from(path),
    }
}

/// Extracts all variable names in the given string that
/// are of the form `<varname>`. The surrounding brackets
/// are also returned.
fn parse_env_variables(input: &str) -> Vec<&str> {
    let mut vars = vec![];
    let mut depth = 0;
    let mut start_index = 0;

    for (index, ch) in input.chars().enumerate() {
        if ch == '<' {
            start_index = index;
            depth += 1;
        } else if ch == '>' && depth > 0 {
            if depth > 0 {
                vars.push(&input[start_index..=index]);
            }
            depth -= 1;
        }
    }

    vars
}

fn resolve_tendril_bundle(
    td_bundle: &TendrilBundle,
    first_only: bool
) -> Vec<Result<Tendril, InvalidTendrilError>> {
    let mode = match (&td_bundle.dir_merge, &td_bundle.link) {
        (true, false) => TendrilMode::DirMerge,
        (false, false) => TendrilMode::DirOverwrite,
        (_, true) => TendrilMode::Link,
    };

    let raw_paths = match (first_only, td_bundle.parents.is_empty()) {
        (true, false) => vec![td_bundle.parents[0].clone()],
        (false, false) => td_bundle.parents.clone(),
        (_, true) => vec![],
    };

    let mut resolve_results = 
        Vec::with_capacity(td_bundle.names.len() * td_bundle.parents.len());

    for name in td_bundle.names.iter() {
        for raw_path in raw_paths.iter() {
            let parent = resolve_path_variables(String::from(raw_path));

            resolve_results.push(Tendril::new(
                &td_bundle.group,
                name,
                parent,
                mode.clone(),
            ));
        }
    }

    resolve_results
}

/// Returns `true` if the current *Tendrils* process is capable
/// of creating symlinks.
/// 
/// This is mainly applicable on Windows, where creating symlinks
/// requires administrator priviledges, or enabling *Developer Mode*.
/// On Unix platforms this always returns `true`.
pub fn can_symlink() -> bool {
    #[cfg(windows)]
    match std::env::consts::FAMILY {
        "windows" => is_root::is_root() || is_dev_mode(),
        _ => true,
    }

    #[cfg(unix)]
    true
}

/// Returns `true` if *Developer Mode* is enabled on Windows.
/// Returns `false` if the setting cannot be determined for any reason.
#[cfg(windows)]
fn is_dev_mode() -> bool {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let app_model = match hklm.open_subkey(
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AppModelUnlock"
    ) {
        Ok(v) => v,
        _ => return false,
    };

    let reg_value: u32 = match app_model.get_value(
        "AllowDevelopmentWithoutDevLicense"
    ) {
        Ok(v) => v,
        _ => return false,
    };

    reg_value == 1
}

fn symlink(
    create_at: &Path, target: &Path, dry_run: bool, force: bool
) -> Result<TendrilActionSuccess, TendrilActionError> {
    if !force && target.is_symlink() {
        Err(TendrilActionError::TypeMismatch {
            loc: Location::Source,
            mistype: FsoType::Symlink,
        })
    }
    else if target.exists() {
        match create_dir_all(create_at.parent().unwrap_or(create_at)) {
            Err(e) => return Err(TendrilActionError::IoError {
                kind: e.kind(),
                loc: Location::Dest,
            }),
            _ => {},
        };

        #[cfg(windows)]
        return symlink_win(create_at, target, dry_run);
        #[cfg(unix)]
        return symlink_unix(create_at, target, dry_run);
    }
    else {
        Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::NotFound,
            loc: Location::Source,
        })
    }
}

#[cfg(unix)]
fn symlink_unix(
    create_at: &Path, target: &Path, dry_run: bool
) -> Result<TendrilActionSuccess, TendrilActionError> {
    let create_at_existed = create_at.exists();
    match (dry_run, create_at_existed) {
        (true, true) => return Ok(TendrilActionSuccess::OverwriteSkipped),
        (true, false) => return Ok(TendrilActionSuccess::NewSkipped),
        (false, true) => {
            if create_at.is_file() {
                remove_file(create_at)?;
            }
            else {
                remove_dir_all(create_at)?;
            }
        },
        _ => {}
    };
    match (std::os::unix::fs::symlink(target, create_at), create_at_existed) {
        (Ok(_), true) => Ok(TendrilActionSuccess::Overwrite),
        (Ok(_), false) => Ok(TendrilActionSuccess::New),
        (Err(e), _) => Err(TendrilActionError::from_io_err(
            e,
            Location::Unknown,
        )),
    }
}

#[cfg(windows)]
fn symlink_win(
    create_at: &Path, target: &Path, dry_run: bool
) -> Result<TendrilActionSuccess, TendrilActionError> {
    use std::os::windows::fs::{symlink_dir, symlink_file};

    let create_at_existed = create_at.exists();
    match (dry_run, create_at_existed) {
        (true, true) => return Ok(TendrilActionSuccess::OverwriteSkipped),
        (true, false) => return Ok(TendrilActionSuccess::NewSkipped),
        (false, true) => {
            if create_at.is_file() {
                match remove_file(create_at) {
                    Err(e) => return Err(
                        TendrilActionError::IoError {
                            kind: e.kind(),
                            loc: Location::Dest,
                        }
                    ),
                    _ => {}
                };
            }
            else {
                remove_dir_all(create_at)?;
            }
        },
        _ => {}
    }

    let result;
    if target.is_dir() {
        result = symlink_dir(target, create_at);
    }
    else {
        result = symlink_file(target, create_at);
    }

    match (result, create_at_existed) {
        (Ok(_), true) => Ok(TendrilActionSuccess::Overwrite),
        (Ok(_), false) => Ok(TendrilActionSuccess::New),
        (Err(e), _) => Err(TendrilActionError::IoError {
            kind: e.kind(),
            loc: Location::Dest,
        })
    }
}

/// Performs a tendril action on the list of given tendrils and returns
/// reports for each action.
/// 
/// # Arguments
/// - `mode` - The action mode to be performed.
/// - `td_dir` - The *Tendrils* folder to perform the actions on.
/// - `td_bundles` - The list of tendril bundles to perform the actions on.
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
///     - `false` will simply return [`TendrilActionError::TypeMismatch`] if there
/// is a type mismatch.
/// 
/// # Returns
/// A [`TendrilActionReport`] for each tendril action. A given [`TendrilBundle`] may
/// result in many actions if it includes multiple names and/or parents.
/// The order of the actions & reports maintains the order of the given
/// tendrils, but each one is expanded into individual tendrils firstly by
/// each of its `names`, then by each of its `parents`. For example, for a
/// list of two tendrils [t1, t2], each having multiple names [n1, n2] and
/// multiple parents [p1, p2], the list will be expanded to:
/// - t1_n1_p1
/// - t1_n1_p2
/// - t1_n2_p1
/// - t1_n2_p2
/// - t2_n1_p1
/// - t2_n1_p2
/// - t2_n2_p1
/// - t2_n2_p2
pub fn tendril_action<'a>(
    mode: ActionMode,
    td_dir: &Path,
    td_bundles: &'a [TendrilBundle],
    dry_run: bool,
    force: bool,
) -> Vec<TendrilActionReport<'a>> {
    let mut action_reports: Vec<TendrilActionReport> = vec![];
    let first_only = mode == ActionMode::Pull;
    let can_symlink = mode == ActionMode::Link && can_symlink();

    for tendril in td_bundles.iter() {
        let resolved_tendrils = resolve_tendril_bundle(&tendril, first_only);

        // The number of parents that were considered when
        // resolving the tendril bundle
        let num_parents = match first_only {
            true => 1,
            false => tendril.parents.len(),
        };

        for (i, resolved_tendril) in resolved_tendrils.into_iter().enumerate() {
            let action_result = match (&resolved_tendril, &mode, can_symlink) {
                (Ok(v), ActionMode::Pull, _) => {
                    Some(pull_tendril(&td_dir, &v, dry_run, force))
                },
                (Ok(v), ActionMode::Push, _) => {
                    Some(push_tendril(&td_dir, &v, dry_run, force))
                },
                (Ok(v), ActionMode::Link, true) => {
                    Some(link_tendril(&td_dir, &v, dry_run, force))
                },
                (Ok(_v), ActionMode::Link, false) => {
                    // Do not attempt to symlink if it has already been determined
                    // that the process does not have the required permissions.
                    // This prevents deleting any of the remote files unnecessarily.
                    Some(Err(TendrilActionError::IoError {
                        kind: std::io::ErrorKind::PermissionDenied,
                        loc: Location::Dest,
                    }))
                },
                (Err(_), _, _) => None,
            };

            let resolved_path = match resolved_tendril {
                Ok(v) => Ok(v.full_path()),
                Err(e) => Err(e),
            };

            let name_idx = ((i/num_parents) as f32).floor() as usize;

            let report = TendrilActionReport {
                orig_tendril: tendril,
                name: &tendril.names[name_idx],
                resolved_path,
                action_result,
            };
            action_reports.push(report);
        }
    }

    action_reports
}
