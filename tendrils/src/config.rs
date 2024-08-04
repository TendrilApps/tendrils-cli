use crate::get_home_dir;
use crate::tendril_bundle::TendrilBundle;
use crate::enums::GetConfigError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

/// Contains the configuration context for Tendrils.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// The tendrils that are defined in a Tendrils repo.
    #[serde(default)]
    pub tendrils: Vec<TendrilBundle>
}

/// Parses the `tendrils.json` file in the given Tendrils repo and returns
/// the configuration within.
/// The tendril bundles are returned in the order they are defined in the file.
///
/// # Arguments
/// - `td_repo` - Path to the Tendrils folder.
pub fn get_config(
    td_repo: &Path,
) -> Result<Config, GetConfigError> {
    let tendrils_file_path = Path::new(&td_repo).join(".tendrils/tendrils.json");
    let tendrils_file_contents = std::fs::read_to_string(tendrils_file_path)?;
    let tendrils = parse_config(&tendrils_file_contents)?;
    Ok(tendrils)
}

/// See tests at [API level](`crate::TendrilsApi::get_default_repo_path`)
pub fn get_default_repo_path() -> Result<Option<PathBuf>, std::io::Error> {
    let home_dir = match get_home_dir() {
        Some(v) => v,
        None => return Ok(None),
    };
    let config_file = PathBuf::from(home_dir).join(".tendrils/repo_path");
    match std::fs::read_to_string(config_file) {
        Ok(v) if v.is_empty() => Ok(None),
        Ok(v) => Ok(Some(PathBuf::from(v))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(None)
        }
        Err(e) => Err(e.into()),
    }
}

/// # Arguments
/// - `json` - JSON object following the tendrils.json schema
pub(crate) fn parse_config(
    json: &str
) -> Result<Config, serde_json::Error> {
    serde_json::from_str::<Config>(json)
}
