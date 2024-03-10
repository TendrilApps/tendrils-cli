//! Provides tools for managing tendrils.
//! See also the [`td` CLI](..//td/index.html)

mod enums;
pub use enums::{
    ActionMode,
    GetTendrilsError,
    InvalidTendrilError,
    TendrilActionError,
    TendrilActionSuccess,
};
mod resolved_tendril;
pub use resolved_tendril::{
    ResolvedTendril,
    TendrilMode,
};
use std::ffi::OsString;
use std::fs::{create_dir_all, remove_dir_all, remove_file};
use std::path::{Path, PathBuf};
mod tendril;
pub use tendril::Tendril;
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
    to: &Path,
    dir_merge: bool,
    dry_run: bool,
) -> Result<TendrilActionSuccess, TendrilActionError> {
    let mut to = to;

    if from.is_dir() {
        if dry_run { return Ok(TendrilActionSuccess::Skipped); }
        if !dir_merge && to.is_dir() {
            std::fs::remove_dir_all(to)?;
            create_dir_all(to)?;
        }
        else if to.is_file() {
            remove_file(&to)?;
        }

        // TODO: Eliminate this unwrap and test how
        // root folders are handled
        to = to.parent().unwrap();
        create_dir_all(to)?;

        let mut copy_opts = fs_extra::dir::CopyOptions::new();
        copy_opts.overwrite = true;
        copy_opts.skip_exist = false;
        match fs_extra::dir::copy(from, to, &copy_opts) {
            Ok(_v) => Ok(TendrilActionSuccess::Ok),
            Err(e) => match e.kind {
                // Convert fs_extra::errors to PushPullErrors
                fs_extra::error::ErrorKind::Io(e) => {
                    Err(TendrilActionError::from(e))
                },
                fs_extra::error::ErrorKind::PermissionDenied => {
                    let e = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
                    Err(TendrilActionError::from(e))
                },
                _ => {
                    let e = std::io::Error::from(std::io::ErrorKind::Other);
                    Err(TendrilActionError::from(e))
                }
            }
        }
    }
    else if from.is_file() {
        let from_str = match from.to_str() {
            Some(v) => v,
            None => {
                let e = std::io::Error::from(std::io::ErrorKind::InvalidInput);
                return Err(TendrilActionError::from(e))
            }
        };
        let to_str = match to.to_str() {
            Some(v) => v,
            None => {
                let e = std::io::Error::from(std::io::ErrorKind::InvalidInput);
                return Err(TendrilActionError::from(e))
            }
        };

        if dry_run { return Ok(TendrilActionSuccess::Skipped); }

        // TODO: Eliminate this unwrap and test how
        // root folders are handled
        create_dir_all(to.parent().unwrap())?;

        if to.is_dir() {
            remove_dir_all(&to)?;
        }
        else if to.is_symlink() {
            remove_file(&to)?;
        }

        match std::fs::copy(from_str, to_str) {
            Ok(_v) => Ok(TendrilActionSuccess::Ok),
            Err(e) => Err(TendrilActionError::from(e))
        }
    }
    else {
        let e = std::io::Error::from(std::io::ErrorKind::NotFound);
        Err(TendrilActionError::from(e))
    }
}

/// Returns `true` if the type (file vs folder) of the source and
/// destination are mismatched.
/// - Returns `false` if the source or destination do not exist.
/// - Returns `true` if either the source or destination are symlinks.
///
/// Note: This is not useful in link mode.
fn fso_types_mismatch(source: &Path, dest: &Path) -> bool {
    (source.is_dir() && dest.is_file())
        || (source.is_file() && dest.is_dir())
        || source.is_symlink()
        || dest.is_symlink()
}

/// Returns only link style tendrils if the action mode is `Link`, otherwise
/// it returns only the push/pull style tendrils.
///
/// # Arguments
/// - `tendrils` - List of tendrils to filter through.
/// - `mode` - The action to be filtered for (link, push, pull, etc.)
pub fn filter_by_mode(tendrils: Vec<Tendril>, mode: ActionMode) -> Vec<Tendril> {
    tendrils.into_iter()
        .filter(|t| t.link == (mode == ActionMode::Link))
        .collect()
}

/// Returns only tendrils that match any of the given profiles, and those
/// tendrils that belong to all profiles (i.e. those that do not have any
/// profiles defined).
///
/// # Arguments
/// - `tendrils` - List of tendrils to filter through.
/// - `profiles` - The profiles to be filtered for.
pub fn filter_by_profiles(tendrils: Vec<Tendril>, profiles: &[String]) -> Vec<Tendril> {
    if profiles.is_empty() {
        return tendrils;
    }

    tendrils.into_iter().filter(|t| -> bool {
            t.profiles.is_empty()
            || profiles.iter().any(|p| t.profiles.contains(p))
        }).collect()
}

/// Parses the `tendrils.json` file in the given *Tendrils* folder
/// and returns the tendrils in the order they are defined in the file.
///
/// # Arguments
/// - `td_dir` - Path to the *Tendrils* folder.
pub fn get_tendrils(
    td_dir: &Path,
) -> Result<Vec<Tendril>, GetTendrilsError> {
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
    tendril: &ResolvedTendril,
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
    // TODO: Eliminate this unwrap and test with root folders
    if !dest.parent().unwrap().exists() {
        let io_err = std::io::Error::from(std::io::ErrorKind::NotFound);
        return Err(TendrilActionError::IoError(io_err));
    }

    let target = td_dir.join(tendril.group()).join(tendril.name());

    if !force && dest.exists() && !dest.is_symlink() {
        return Err(TendrilActionError::TypeMismatch);
    }

    Ok(symlink(&dest, &target, dry_run, force)?)
}

/// # Arguments
/// - `json` - JSON array of Tendrils
fn parse_tendrils(json: &str) -> Result<Vec<Tendril>, serde_json::Error> {
    serde_json::from_str::<Vec<Tendril>>(json)
}

fn pull_tendril(
    td_dir: &Path,
    tendril: &ResolvedTendril,
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

    let dest = td_dir.join(tendril.group()).join(tendril.name());

    if !force && fso_types_mismatch(&source, &dest){
        return Err(TendrilActionError::TypeMismatch);
    }

    let dir_merge = tendril.mode == TendrilMode::DirMerge;
    Ok(copy_fso(&source, &dest, dir_merge, dry_run)?)
}

fn push_tendril(
    td_dir: &Path,
    tendril: &ResolvedTendril,
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
    // TODO: Eliminate this unwrap and test with root folders
    if !dest.parent().unwrap().exists() {
        let io_err = std::io::Error::from(std::io::ErrorKind::NotFound);
        return Err(TendrilActionError::IoError(io_err));
    }

    let source = td_dir.join(tendril.group()).join(tendril.name());

    if !force && fso_types_mismatch(&dest, &source) {
        return Err(TendrilActionError::TypeMismatch);
    }

    let dir_merge = tendril.mode == TendrilMode::DirMerge;
    Ok(copy_fso(&source, &dest, dir_merge, dry_run)?)
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

fn resolve_tendril(
    tendril: &Tendril,
    first_only: bool
) -> Vec<Result<ResolvedTendril, InvalidTendrilError>> {
    let mode = match (&tendril.dir_merge, &tendril.link) {
        (true, false) => TendrilMode::DirMerge,
        (false, false) => TendrilMode::DirOverwrite,
        (_, true) => TendrilMode::Link,
    };

    let raw_paths = match (first_only, tendril.parents.is_empty()) {
        (true, false) => vec![tendril.parents[0].clone()],
        (false, false) => tendril.parents.clone(),
        (_, true) => vec![],
    };

    let mut resolve_results = 
        Vec::with_capacity(tendril.names.len() * tendril.parents.len());

    for name in tendril.names.iter() {
        for raw_path in raw_paths.iter() {
            let parent = resolve_path_variables(String::from(raw_path));

            resolve_results.push(ResolvedTendril::new(
                &tendril.group,
                name,
                parent,
                mode,
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
/// Returns `false` if the setting cannot be determined for whatever reason.
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
        Err(TendrilActionError::TypeMismatch)
    }
    else if target.exists() {
        #[cfg(windows)]
        return symlink_win(create_at, target, dry_run);
        #[cfg(unix)]
        return symlink_unix(create_at, target, dry_run);
    }
    else {
        let io_err = std::io::Error::from(std::io::ErrorKind::NotFound);
        Err(TendrilActionError::IoError(io_err))
    }
}

#[cfg(unix)]
fn symlink_unix(
    create_at: &Path, target: &Path, dry_run: bool
) -> Result<TendrilActionSuccess, TendrilActionError> {
    if dry_run {
        Ok(TendrilActionSuccess::Skipped)
    }
    else {
        if create_at.is_file() {
            remove_file(create_at)?;
        }
        if create_at.is_dir() {
            remove_dir_all(create_at)?;
        }
        std::os::unix::fs::symlink(target, create_at)?;
        Ok(TendrilActionSuccess::Ok)
    }
}

#[cfg(windows)]
fn symlink_win(
    create_at: &Path, target: &Path, dry_run: bool
) -> Result<TendrilActionSuccess, TendrilActionError> {
    use std::os::windows::fs::{symlink_dir, symlink_file};
    // TODO: Pattern match instead
    if target.is_dir() {
        if dry_run {
            return Ok(TendrilActionSuccess::Skipped);
        }
        else if create_at.exists() {
            remove_dir_all(create_at)?;
        }
        symlink_dir(target, create_at)?;
        Ok(TendrilActionSuccess::Ok)
    }
    else if target.is_file() {
        if dry_run {
            return Ok(TendrilActionSuccess::Skipped);
        }
        else if create_at.exists() {
            remove_file(create_at)?;
        }
        symlink_file(target, create_at)?;
        Ok(TendrilActionSuccess::Ok)
    }
    else {
        let io_err = std::io::Error::from(std::io::ErrorKind::NotFound);
        Err(TendrilActionError::IoError(io_err))
    }
}

/// Performs a tendril action on the list of given tendrils and returns
/// reports for each action.
/// 
/// # Arguments
/// - `mode` - The action mode to be performed.
/// - `td_dir` - The *Tendrils* folder to perform the actions on.
/// - `tendrils` - The list of tendrils to perform the actions on.
/// - `dry_run`
///     - `true` will perform the internal checks for the action but does not
/// modify anything on the file system. If the action is expected to fail, the
/// expected [`TendrilActionError`] is returned. If it's expected to succeed,
/// it returns [`TendrilActionSuccess::Skipped`]. Note: It is still possible
/// for a successful dry run to fail in an actual run.
///     - `false` will perform the action normally (modifying the file system),
/// and will return [`TendrilActionSuccess::Ok`] if successful.
/// - `force`
///     - `true` will ignore any type mismatches and will force the operation.
///     - `false` will simply return [`TendrilActionError::TypeMismatch`] if there
/// is a type mismatch.
/// 
/// # Returns
/// A [`TendrilActionReport`] for each tendril action. A given [`Tendril`] may
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
    tendrils: &'a [Tendril],
    dry_run: bool,
    force: bool,
) -> Vec<TendrilActionReport<'a>> {
    let mut action_reports: Vec<TendrilActionReport> = vec![];
    let first_only = mode == ActionMode::Pull;
    let can_symlink = mode == ActionMode::Link && can_symlink();

    for tendril in tendrils.iter() {
        let resolved_tendrils = resolve_tendril(&tendril, first_only);

        // The number of parents that were considered when
        // resolving the tendril bundle
        let num_parents = match first_only {
            true => 1,
            false => tendril.parents.len(),
        };

        for (i, resolved_tendril) in resolved_tendrils.into_iter().enumerate() {
            let action_result = match (&resolved_tendril, mode, can_symlink) {
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
                    let io_err = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
                    Some(Err(TendrilActionError::IoError(io_err)))
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
