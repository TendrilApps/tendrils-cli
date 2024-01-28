mod action_mode;
use action_mode::ActionMode;
pub mod cli;
mod errors;
use errors::{
    GetTendrilsError,
    TendrilActionError,
    ResolveTendrilError,
};
mod resolved_tendril;
use resolved_tendril::{
    ResolvedTendril,
    TendrilMode,
};
use std::fs::{create_dir_all, remove_file};
use std::path::{Path, PathBuf};
mod tendril;
use tendril::Tendril;
mod tendril_action_report;
use tendril_action_report::TendrilActionReport;

#[cfg(test)]
mod libtests;
#[cfg(test)]
mod test_utils;
#[cfg(test)]
use test_utils::get_mut_testing_var;

fn copy_fso(
    from: &Path,
    to: &Path,
    folder_merge: bool,
    dry_run: bool,
) -> Result<(), TendrilActionError> {
    let mut to = to;

    if from.is_dir() {
        if dry_run { return Err(TendrilActionError::Skipped); }
        if !folder_merge && to.exists() {
            std::fs::remove_dir_all(to)?;
            create_dir_all(to)?;
        }
        // TODO: Eliminate this unwrap and test how
        // root folders are handled
        to = to.parent().unwrap();
        create_dir_all(to)?;

        let mut copy_opts = fs_extra::dir::CopyOptions::new();
        copy_opts.overwrite = true;
        copy_opts.skip_exist = false;
        match fs_extra::dir::copy(from, to, &copy_opts) {
            Ok(_v) => Ok(()),
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

        if dry_run { return Err(TendrilActionError::Skipped); }

        // TODO: Eliminate this unwrap and test how
        // root folders are handled
        create_dir_all(to.parent().unwrap())?;

        match std::fs::copy(from_str, to_str) {
            Ok(_v) => Ok(()),
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
fn get_tendrils_folder(starting_path: &Path) -> Option<PathBuf> {
    if is_tendrils_folder(starting_path) {
        Some(starting_path.to_owned())
    }
    else {
        match std::env::var("TENDRILS_FOLDER") {
            Ok(v) => {
                let test_path = PathBuf::from(v);
                if is_tendrils_folder(&test_path) {
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
    tendrils_folder: &Path,
) -> Result<Vec<Tendril>, GetTendrilsError> {
    let tendrils_file_path = Path::new(&tendrils_folder).join("tendrils.json");
    let tendrils_file_contents = std::fs::read_to_string(tendrils_file_path)?;
    let tendrils = parse_tendrils(&tendrils_file_contents)?;
    Ok(tendrils)
}

fn get_tendril_overrides(
    tendrils_folder: &Path,
) -> Result<Vec<Tendril>, GetTendrilsError> {
    let tendrils_file_path =
        Path::new(&tendrils_folder).join("tendrils-override.json");

    let tendrils_file_contents = if tendrils_file_path.is_file() {
        std::fs::read_to_string(tendrils_file_path)?
    }
    else {
        return Ok([].to_vec());
    };

    let tendrils = parse_tendrils(&tendrils_file_contents)?;
    Ok(tendrils)
}

fn get_username() -> Result<String, std::env::VarError> {
    match std::env::consts::OS {
        "macos" => Ok(std::env::var("USER")?),
        "windows" => Ok(std::env::var("USERNAME")?),
        _ => unimplemented!()
    }
}

fn is_tendrils_folder(dir: &Path) -> bool {
    dir.join("tendrils.json").is_file()
}

fn is_recursive_tendril(
    tendrils_folder: &Path,
    tendril_full_path: &Path,
) -> bool {
    tendrils_folder == tendril_full_path
        || tendrils_folder.ancestors().any(|p| p == tendril_full_path)
        || tendril_full_path.ancestors().any(|p| p == tendrils_folder)
}

fn link_tendril(
    tendrils_folder: &Path,
    tendril: &ResolvedTendril,
    dry_run: bool,
) -> Result<(), TendrilActionError> {
    let dest= tendril.full_path();
    if tendril.mode != TendrilMode::Link {
        return Err(TendrilActionError::ModeMismatch);
    }
    else if is_recursive_tendril(tendrils_folder, &dest) {
        return Err(TendrilActionError::Recursion);
    }

    let target = tendrils_folder.join(tendril.app()).join(tendril.name());

    if dest.exists() && !dest.is_symlink() {
        return Err(TendrilActionError::TypeMismatch);
    }

    Ok(symlink(&dest, &target, dry_run)?)
}

/// # Arguments
/// - `json` - JSON array of Tendrils
fn parse_tendrils(json: &str) -> Result<Vec<Tendril>, serde_json::Error> {
    serde_json::from_str::<Vec<Tendril>>(json)
}

fn pull_tendril(
    tendrils_folder: &Path,
    tendril: &ResolvedTendril,
    dry_run: bool,
) -> Result<(), TendrilActionError> {
    let source= tendril.full_path();
    if tendril.mode == TendrilMode::Link {
        return Err(TendrilActionError::ModeMismatch);
    }
    else if is_recursive_tendril(tendrils_folder, &source) {
        return Err(TendrilActionError::Recursion);
    }

    let dest = tendrils_folder.join(tendril.app()).join(tendril.name());

    if fso_types_mismatch(&source, &dest) {
        return Err(TendrilActionError::TypeMismatch);
    }

    let folder_merge = tendril.mode == TendrilMode::FolderMerge;
    Ok(copy_fso(&source, &dest, folder_merge, dry_run)?)
}

fn push_tendril(
    tendrils_folder: &Path,
    tendril: &ResolvedTendril,
    dry_run: bool,
) -> Result<(), TendrilActionError> {
    let dest= tendril.full_path();
    if tendril.mode == TendrilMode::Link {
        return Err(TendrilActionError::ModeMismatch);
    }
    if is_recursive_tendril(tendrils_folder, &dest) {
        return Err(TendrilActionError::Recursion);
    }

    let source = tendrils_folder.join(tendril.app()).join(tendril.name());

    if fso_types_mismatch(&dest, &source) {
        return Err(TendrilActionError::TypeMismatch);
    }

    let folder_merge = tendril.mode == TendrilMode::FolderMerge;
    Ok(copy_fso(&source, &dest, folder_merge, dry_run)?)
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

fn resolve_path_variables(path: &Path) -> Result<PathBuf, ResolveTendrilError> {
    let orig_string = match path.to_str() {
        Some(v) => v,
        None => return Err(ResolveTendrilError::PathParseError)
    };

    // TODO: Extract var sets as a constant expression?
    let supported_var_sets: &[(&str, fn() -> Result<String, std::env::VarError>)] = &[
        ("<user>", get_username),
        #[cfg(test)]
        ("<mut-testing>", get_mut_testing_var),
    ];

    let mut resolved: String = orig_string.to_string();
    for var_set in supported_var_sets {
        let value = var_set.1().unwrap_or(var_set.0.to_string());
        resolved = resolved.replace(var_set.0, &value);
    }

    Ok(PathBuf::from(&resolved))
}

fn resolve_tendril(
    tendril: Tendril, // TODO: Use reference only?
    first_only: bool
) -> Vec<Result<ResolvedTendril, ResolveTendrilError>> {
    let mode = match (&tendril.folder_merge, &tendril.link) {
        (true, false) => TendrilMode::FolderMerge,
        (false, false) => TendrilMode::FolderOverwrite,
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

    raw_paths.iter().map(|p| -> Result<ResolvedTendril, ResolveTendrilError> {
        let parent = resolve_path_variables(&PathBuf::from(p))?;

        Ok(ResolvedTendril::new(
            tendril.app.clone(),
            tendril.name.clone(),
            parent,
            mode,
        )?)
    }).collect()
}

fn symlink(create_at: &Path, target: &Path, dry_run: bool) -> Result<(), TendrilActionError> {
    // TODO: Eliminate this unwrap and test with root folders
    if !create_at.parent().unwrap().exists() {
        let io_err = std::io::Error::from(std::io::ErrorKind::NotFound);
        Err(TendrilActionError::IoError(io_err))
    }
    else if target.is_symlink()
        || (create_at.exists() && !create_at.is_symlink()) {
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
fn symlink_unix(create_at: &Path, target: &Path, dry_run: bool) -> Result<(), TendrilActionError> {
    if dry_run {
        Err(TendrilActionError::Skipped)
    }
    else {
        if create_at.exists() {
            remove_file(create_at)?;
        }
        Ok(std::os::unix::fs::symlink(target, create_at)?)
    }
}

#[cfg(windows)]
fn symlink_win(create_at: &Path, target: &Path, dry_run: bool) -> Result<(), TendrilActionError> {
    use std::os::windows::fs::{symlink_dir, symlink_file};
    // TODO: Pattern match instead
    if target.is_dir() {
        if dry_run {
            return Err(TendrilActionError::Skipped);
        }
        else if create_at.exists() {
            remove_file(create_at)?;
        }
        Ok(symlink_dir(target, create_at)?)
    }
    else if target.is_file() {
        if dry_run {
            return Err(TendrilActionError::Skipped);
        }
        else if create_at.exists() {
            remove_file(create_at)?;
        }
        Ok(symlink_file(target, create_at)?)
    }
    else {
        Err(TendrilActionError::TypeMismatch)
    }
}

pub fn tendril_action<'a>(
    mode: ActionMode,
    tendrils_folder: &Path,
    tendrils: &'a [Tendril],
    dry_run: bool,
) -> Vec<TendrilActionReport<'a>> {
    let mut action_reports: Vec<TendrilActionReport> = vec![];
    let first_only = mode == ActionMode::Pull;

    for tendril in tendrils.iter() {
        let resolve_results = resolve_tendril(tendril.clone(), first_only);
        let mut action_results = vec![];
        for result in resolve_results.iter() {
            match (result, mode) {
                (Ok(v), ActionMode::Pull) => {
                    action_results.push(Some(pull_tendril(&tendrils_folder, &v, dry_run)));
                },
                (Ok(v), ActionMode::Push) => {
                    action_results.push(Some(push_tendril(&tendrils_folder, &v, dry_run)));
                },
                (Ok(v), ActionMode::Link) => {
                    action_results.push(Some(link_tendril(&tendrils_folder, &v, dry_run)));
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
