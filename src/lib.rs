mod errors;
use errors::{GetTendrilsError, PushPullError, ResolvePathError};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
mod tendril;
use tendril::Tendril;

#[cfg(test)]
mod libtests;

fn copy_fso(
    from: &Path,
    to: &Path,
    folder_merge: bool
) -> Result<(), std::io::Error> {
    let mut to = to;

    if from.is_dir() {
        if !folder_merge {
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
                // Convert fs_extra::errors to std::io::errors
                fs_extra::error::ErrorKind::Io(e) => {
                    Err(e)
                },
                fs_extra::error::ErrorKind::PermissionDenied => {
                    let e = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
                    Err(e)
                },
                _ => Err(std::io::Error::from(std::io::ErrorKind::Other))
            }
        }
    }
    else if from.is_file() {
        // TODO: Eliminate this unwrap and test how
        // root folders are handled
        create_dir_all(to.parent().unwrap())?;

        let from_str = match from.to_str() {
            Some(v) => v,
            None => return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput))
        };
        let to_str = match to.to_str() {
            Some(v) => v,
            None => return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput))
        };
        match std::fs::copy(from_str, to_str) {
            Ok(_v) => Ok(()),
            Err(e) => Err(e)
        }
    }
    else {
        return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
    }
}

// TODO: Recursively look through all parent folders before
// checking environment variable
pub fn get_tendrils_folder(starting_path: &Path) -> Option<PathBuf> {
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

pub fn get_tendrils(
    tendrils_folder: &Path,
) -> Result<Vec<Tendril>, GetTendrilsError> {
    let tendrils_file_path = Path::new(&tendrils_folder).join("tendrils.json");
    let tendrils_file_contents = std::fs::read_to_string(tendrils_file_path)?;
    let tendrils = parse_tendrils(&tendrils_file_contents)?;
    Ok(tendrils)
}

pub fn get_tendril_overrides(
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

fn is_path(x: &str) -> bool {
    x.contains('/') || x.contains('\\')
}

pub fn is_tendrils_folder(dir: &Path) -> bool {
    dir.join("tendrils.json").is_file()
}

/// # Arguments
/// - `json` - JSON array of Tendrils
fn parse_tendrils(json: &str) -> Result<Vec<Tendril>, serde_json::Error> {
    serde_json::from_str::<Vec<Tendril>>(json)
}

pub fn pull<'a>(
    tendrils_folder: &Path,
    tendrils: &'a [Tendril],
) -> Vec<(&'a Tendril, Result<(), PushPullError>)> {
    let mut results = Vec::with_capacity(tendrils.len());
    let mut ids: Vec<String> = Vec::with_capacity(tendrils.len());
    
    for tendril in tendrils {
        let id = tendril.id();

        let result = match ids.contains(&id) {
            true => Err(PushPullError::Duplicate),
            false => pull_tendril(tendrils_folder, tendril)
        };

        ids.push(id);
        results.push((tendril, result));
    }

    results
}

fn pull_tendril(
    tendrils_folder: &Path,
    tendril: &Tendril,
) -> Result<(), PushPullError> {
    if tendril.app.is_empty()
        || tendril.name.is_empty()
        || tendril.app.to_lowercase() == ".git"
        || is_path(&tendril.app)
        || is_path(&tendril.name){
        return Err(PushPullError::InvalidId);
    }

    // TODO: Consider conditional compilation instead
    // of matching on every iteration
    // TODO: Extract this path determination to a separate
    // function to use with push as well
    let sources = match std::env::consts::OS {
        "macos" => &tendril.parent_dirs_mac,
        "windows" => &tendril.parent_dirs_windows,
        _ => return Err(PushPullError::Unsupported)
    };

    if sources.is_empty() {
        return Err(PushPullError::Skipped);
    }

    let source= resolve_path_variables(&PathBuf::from(&sources[0]))?
        .join(&tendril.name);
    if tendrils_folder == source 
        || tendrils_folder.ancestors().any(|p| p == source)
        || source.ancestors().any(|p| p == tendrils_folder) {
        return Err(PushPullError::Recursion);
    }

    let dest = tendrils_folder.join(&tendril.app).join(&tendril.name);

    if (source.is_dir() && dest.is_file())
        || (source.is_file() && dest.is_dir())
        || source.is_symlink()
        || dest.is_symlink() {
        return Err(PushPullError::TypeMismatch);
    }

    Ok(copy_fso(&source, &dest, tendril.folder_merge)?)
}

/// Returns a list of all Tendrils after replacing global ones with any
/// applicable overrides.
/// # Arguments
/// - `global` - The set of Tendrils (typically defined in tendrils.json)
/// - `overrides` - The set of Tendril overrides (typically defined in
///   tendrils-overrides.json)
pub fn resolve_overrides(
    global: &[Tendril],
    overrides: &[Tendril],
) -> Vec<Tendril> {
    let mut resolved_tendrils = Vec::with_capacity(global.len());

    for tendril in global {
        let mut last_index: usize = 0;
        let overrides_iter = overrides.iter();

        if overrides_iter.enumerate().any(|(i, x)| {
            last_index = i;
            x.id() == tendril.id() })
        {
            resolved_tendrils.push(overrides[last_index].clone());
        }
        else {
            resolved_tendrils.push(tendril.clone())
        }
    }

    resolved_tendrils
}

fn resolve_path_variables(path: &Path) -> Result<PathBuf, ResolvePathError> {
    let orig_string = match path.to_str() {
        Some(v) => v,
        None => return Err(ResolvePathError::PathParseError)
    };

    let username = match std::env::consts::OS {
        "macos" => std::env::var("USER")?,
        "windows" => std::env::var("USERNAME")?,
        _ => "<user>".to_string()
    };

    let resolved = orig_string.replace("<user>", &username);
    Ok(PathBuf::from(&resolved))
}
