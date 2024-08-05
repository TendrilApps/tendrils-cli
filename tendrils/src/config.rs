use crate::{get_home_dir, ConfigType};
use crate::tendril_bundle::TendrilBundle;
use crate::enums::GetConfigError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

/// Contains the configuration context for a Tendrils repo.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// The tendrils that are defined in a Tendrils repo.
    #[serde(default)]
    pub tendrils: Vec<TendrilBundle>
}

/// Contains the global configuration context for Tendrils.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalConfig {
    /// The path to the default Tendrils repo.
    #[serde(rename = "default-repo-path")]
    pub default_repo_path: Option<PathBuf>,
}

impl GlobalConfig {
    fn new() -> GlobalConfig {
        GlobalConfig {
            default_repo_path: None,
        }
    }
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
    let config_file_path = td_repo.join(".tendrils/tendrils.json");
    let config_file_contents = std::fs::read_to_string(config_file_path)?;
    let config = parse_config(&config_file_contents)?;
    Ok(config)
}

/// Parses the `~/.tendrils/global-config.json` file and returns the
/// configuration within. If the file doesn't exist, an empty configuration is
/// returned (i.e all fields set to `None`).
pub fn get_global_config() -> Result<GlobalConfig, GetConfigError> {
    let home_dir = match get_home_dir() {
        Some(v) => v,
        None => return Ok(GlobalConfig::new()),
    };
    let config_file_path = PathBuf::from(home_dir).join(".tendrils/global-config.json");
    let config_file_contents = match std::fs::read_to_string(config_file_path) {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(GlobalConfig::new())
        }
        Err(e) => return Err(
            Into::<GetConfigError>::into(e).with_cfg_type(ConfigType::Global)
        )
    };

    match parse_global_config(&config_file_contents) {
        Ok(v) => Ok(v),
        Err(e) => Err(
            Into::<GetConfigError>::into(e).with_cfg_type(ConfigType::Global)
        ),
    }
}

/// # Arguments
/// - `json` - JSON object following the tendrils.json schema
pub(crate) fn parse_config(
    json: &str
) -> Result<Config, serde_json::Error> {
    serde_json::from_str::<Config>(json)
}

/// # Arguments
/// - `json` - JSON object following the global-config.json schema
fn parse_global_config(
    json: &str
) -> Result<GlobalConfig, serde_json::Error> {
    serde_json::from_str::<GlobalConfig>(json)
}
