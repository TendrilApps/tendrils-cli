use crate::tendril_bundle::TendrilBundle;
use crate::enums::GetConfigError;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[cfg(test)]
mod tests;

/// Contains the configuration context for Tendrils.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// The tendrils that are defined in a Tendrils folder.
    pub tendrils: Vec<TendrilBundle>
}

/// Parses the `tendrils.json` file in the given *Tendrils* folder and returns
/// the configuration within.
/// The tendril bundles are returned in the order they are defined in the file.
///
/// # Arguments
/// - `td_dir` - Path to the *Tendrils* folder.
pub fn get_config(
    td_dir: &Path,
) -> Result<Config, GetConfigError> {
    let tendrils_file_path = Path::new(&td_dir).join("tendrils.json");
    let tendrils_file_contents = std::fs::read_to_string(tendrils_file_path)?;
    let tendrils = parse_config(&tendrils_file_contents)?;
    Ok(tendrils)
}

/// # Arguments
/// - `json` - JSON object following the tendrils.json schema
pub(crate) fn parse_config(
    json: &str
) -> Result<Config, serde_json::Error> {
    serde_json::from_str::<Config>(json)
}
