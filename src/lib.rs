mod action_mode;
use action_mode::ActionMode;
pub mod cli;
mod enums;
use enums::{
    GetTendrilsError,
    ResolveTendrilError,
    TendrilActionError,
    TendrilActionSuccess,
};
mod resolved_tendril;
use resolved_tendril::{
    ResolvedTendril,
    TendrilMode,
};
use std::ffi::OsString;
use std::fs::{create_dir_all, remove_dir_all, remove_file};
use std::path::{Path, PathBuf};
mod tendril;
use tendril::Tendril;
mod tendril_action_report;
use tendril_action_report::TendrilActionReport;

#[cfg(test)]
mod libtests;
#[cfg(test)]
mod test_utils;

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
        return Err(TendrilActionError::from(e));
    }
}

fn fso_types_mismatch(source: &Path, dest: &Path) -> bool {
    (source.is_dir() && dest.is_file())
        || (source.is_file() && dest.is_dir())
        || source.is_symlink()
        || dest.is_symlink()
}

// TODO: Recursively look through all parent folders before
// checking environment variable
fn get_tendrils_dir(starting_path: &Path) -> Option<PathBuf> {
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

fn get_tendrils(
    td_dir: &Path,
) -> Result<Vec<Tendril>, GetTendrilsError> {
    let tendrils_file_path = Path::new(&td_dir).join("tendrils.json");
    let tendrils_file_contents = std::fs::read_to_string(tendrils_file_path)?;
    let tendrils = parse_tendrils(&tendrils_file_contents)?;
    Ok(tendrils)
}

fn get_tendril_overrides(
    td_dir: &Path,
) -> Result<Vec<Tendril>, GetTendrilsError> {
    let tendrils_file_path =
        Path::new(&td_dir).join("tendrils-override.json");

    let tendrils_file_contents = if tendrils_file_path.is_file() {
        std::fs::read_to_string(tendrils_file_path)?
    }
    else {
        return Ok([].to_vec());
    };

    let tendrils = parse_tendrils(&tendrils_file_contents)?;
    Ok(tendrils)
}

fn is_tendrils_dir(dir: &Path) -> bool {
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
    else if is_recursive_tendril(td_dir, &dest) {
        return Err(TendrilActionError::Recursion);
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

    let source = td_dir.join(tendril.group()).join(tendril.name());

    if !force && fso_types_mismatch(&dest, &source) {
        return Err(TendrilActionError::TypeMismatch);
    }

    let dir_merge = tendril.mode == TendrilMode::DirMerge;
    Ok(copy_fso(&source, &dest, dir_merge, dry_run)?)
}

/// Returns a list of all Tendrils after replacing global ones with any
/// applicable overrides.
/// # Arguments
/// - `global` - The set of Tendrils (typically defined in tendrils.json)
/// - `overrides` - The set of Tendril overrides (typically defined in
///   tendrils-overrides.json)
fn resolve_overrides(
    global: &[Tendril],
    overrides: &[Tendril],
) -> Vec<Tendril> {
    let mut combined_tendrils = Vec::with_capacity(global.len());

    for tendril in global {
        let mut last_index: usize = 0;
        let overrides_iter = overrides.iter();

        if overrides_iter.enumerate().any(|(i, x)| {
            last_index = i;
            x.id() == tendril.id() })
        {
            combined_tendrils.push(overrides[last_index].clone());
        }
        else {
            combined_tendrils.push(tendril.clone())
        }
    }

    combined_tendrils
}

/// Replaces all environment variables in the format `<varname>` in the
/// given path with their values. If the variable is not found, the
/// `<varname>` is left as-is in the path.
/// 
/// The common tilde (`~`) symbol can also be used as a prefix to the path
/// and corresponds to the `HOME` variable on Unix/Windows.
/// 
/// Any non UTF-8 characters in a variable's value or in the tilde value
/// are replaced with the U+FFFD replacement character.
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

/// Replaces the first instance of `~` with the `HOME` variable (Unix &
/// Windows) and returns the replaced string.
/// 
/// Note: This does *not* check that the tilde is the leading character (it could be
/// anywhere in the string) - this check should be done prior to calling this.
fn resolve_tilde(path: &str) -> String {
    match std::env::var_os("HOME") {
        Some(v) => {
            path.replacen('~', &v.to_string_lossy(), 1)
        },
        None => path.to_string(),
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
    tendril: Tendril, // TODO: Use reference only?
    first_only: bool
) -> Vec<Result<ResolvedTendril, ResolveTendrilError>> {
    let mode = match (&tendril.dir_merge, &tendril.link) {
        (true, false) => TendrilMode::DirMerge,
        (false, false) => TendrilMode::DirOverwrite,
        (_, true) => TendrilMode::Link,
    };
    // TODO: Consider conditional compilation instead
    // of matching on every iteration
    let raw_paths = match std::env::consts::OS {
        "macos" => tendril.parent_dirs_mac.clone(),
        "windows" => tendril.parent_dirs_windows.clone(),
        _ => return vec![]
    };
    let raw_paths = match first_only {
        true => {
            if !raw_paths.is_empty() {
                raw_paths[..1].to_vec()
            }
            else {
                raw_paths
            }
        }
        false => raw_paths
    };

    raw_paths.into_iter().map(|p| -> Result<ResolvedTendril, ResolveTendrilError> {
        let parent = resolve_path_variables(p);

        Ok(ResolvedTendril::new(
            tendril.group.clone(),
            tendril.name.clone(),
            parent,
            mode,
        )?)
    }).collect()
}

fn symlink(
    create_at: &Path, target: &Path, dry_run: bool, force: bool
) -> Result<TendrilActionSuccess, TendrilActionError> {
    // TODO: Eliminate this unwrap and test with root folders
    if !create_at.parent().unwrap().exists() {
        let io_err = std::io::Error::from(std::io::ErrorKind::NotFound);
        Err(TendrilActionError::IoError(io_err))
    }
    else if !force && target.is_symlink() {
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

pub fn tendril_action<'a>(
    mode: ActionMode,
    td_dir: &Path,
    tendrils: &'a [Tendril],
    dry_run: bool,
    force: bool,
) -> Vec<TendrilActionReport<'a>> {
    let mut action_reports: Vec<TendrilActionReport> = vec![];
    let first_only = mode == ActionMode::Pull;

    for tendril in tendrils.iter() {
        let resolve_results = resolve_tendril(tendril.clone(), first_only);
        let mut action_results = vec![];
        for result in resolve_results.iter() {
            match (result, mode) {
                (Ok(v), ActionMode::Pull) => {
                    action_results.push(Some(pull_tendril(&td_dir, &v, dry_run, force)));
                },
                (Ok(v), ActionMode::Push) => {
                    action_results.push(Some(push_tendril(&td_dir, &v, dry_run, force)));
                },
                (Ok(v), ActionMode::Link) => {
                    action_results.push(Some(link_tendril(&td_dir, &v, dry_run, force)));
                },
                (Err(_), _) => action_results.push(None),
            }
        }
        let report = TendrilActionReport {
            orig_tendril: tendril,
            resolved_paths: resolve_results.into_iter().map(|r| {
                match r {
                    Ok(v) => Ok(v.full_path()),
                    Err(e) => Err(e),
                }
            }).collect(),
            action_results,
        };
        action_reports.push(report);
    }
    action_reports
}
