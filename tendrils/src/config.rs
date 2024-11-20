use crate::ConfigType;
use crate::enums::{GetConfigError, OneOrMany, TendrilMode};
use crate::env_ext::get_home_dir;
use crate::path_ext::UniPath;
use crate::tendril::RawTendril;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::path::PathBuf;

#[cfg(test)]
mod tests;

/// Intermediate serialization type for a Tendrils repo configuration.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct SerdeConfig {
    /// The tendrils that are defined in a Tendrils repo.
    /// Using [`IndexMap`](indexmap::IndexMap) to maintain the
    /// order of insertions when iterating over the map.
    #[serde(default)]
    pub tendrils: indexmap::IndexMap<String, OneOrMany<TendrilSet>>
}

/// Contains the configuration context for a Tendrils repo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Config {
    /// The tendrils that are defined in a Tendrils repo.
    pub raw_tendrils: Vec<RawTendril>
}

impl From<SerdeConfig> for Config {
    fn from(serde_cfg: SerdeConfig) -> Self {
        let raw_tendrils = serde_cfg.tendrils.into_iter().map(|(k, v)| {
            let remote_specs: Vec<TendrilSet> = v.into();

            remote_specs.into_iter().map(move |spec| {
                let mode = match (spec.dir_merge, spec.link) {
                    (true, false) => TendrilMode::DirMerge,
                    (false, false) => TendrilMode::DirOverwrite,
                    (_, true) => TendrilMode::Link,
                };

                let local = k.clone();
                let profiles = spec.profiles.clone();
                spec.remotes.into_iter().map(move |r| -> RawTendril {
                    RawTendril {
                        local: local.clone(),
                        remote: r.clone(),
                        mode: mode.clone(),
                        profiles: profiles.clone(),
                    }
                })
            }).flatten()
        }).flatten().collect();

        Config {
            raw_tendrils
        }
    }
}

#[cfg(any(test, feature = "_test_utils"))]
impl From<Config> for SerdeConfig {
    fn from(cfg: Config) -> Self {
        use indexmap::IndexMap;

        let mut tendril_map: IndexMap<String, OneOrMany<TendrilSet>> = indexmap::IndexMap::new();
        for raw in cfg.raw_tendrils.into_iter() {
            let local = raw.local.clone();

            let added_sets: Vec<TendrilSet>;
            // Subsequent inserts are overwriting the value if the keys clash
            if tendril_map.contains_key(&local) {
                let mut exist_sets: Vec<TendrilSet> = tendril_map.get(&local).unwrap().to_owned().into();
                exist_sets.push(raw.into());
                added_sets = exist_sets;
            }
            else {
                added_sets = vec![raw.into()];
            }

            tendril_map.insert(local, added_sets.into());
        }

        SerdeConfig { tendrils: tendril_map }
    }
}

/// Contains the global configuration context for Tendrils.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct GlobalConfig {
    /// The path to the default Tendrils repo.
    #[serde(rename = "default-repo-path")]
    pub default_repo_path: Option<PathBuf>,

    /// The default profiles to be used on this host.
    #[serde(rename = "default-profiles")]
    pub default_profiles: Option<Vec<String>>,
}

impl GlobalConfig {
    fn new() -> GlobalConfig {
        GlobalConfig {
            default_repo_path: None,
            default_profiles: None,
        }
    }
}

pub struct LazyCachedGlobalConfig {
    cached_cfg: Option<Result<GlobalConfig, GetConfigError>>
}

impl LazyCachedGlobalConfig {
    pub fn new() -> LazyCachedGlobalConfig {
        LazyCachedGlobalConfig {
            cached_cfg: None,
        }
    }

    pub fn eval(&mut self) -> Result<GlobalConfig, GetConfigError> {
        match &self.cached_cfg {
            Some(v) => v.clone(),
            None => {
                let global_cfg = get_global_config();
                self.cached_cfg = Some(global_cfg.clone());
                global_cfg
            },
        }
    }

    /// Allows mocking the return value during tests.
    #[cfg(test)]
    pub fn mock_w_parse_err() -> LazyCachedGlobalConfig {
        let mut cfg = Self::new();
        cfg.cached_cfg = Some(Err(GetConfigError::ParseError {
            cfg_type: ConfigType::Global,
            msg: String::from("MOCK VALUE"),
        }));

        cfg
    }
}

/// Intermediate serialization type representing a one-to-many set of tendrils.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct TendrilSet {
    #[serde(deserialize_with = "one_or_many_to_vec")]
    pub remotes: Vec<String>,

    /// `true` indicates that each tendril will have
    /// [`crate::TendrilMode::DirMerge`]. `false` indicates
    /// [`crate::TendrilMode::DirOverwrite`]. Note: this field
    /// may be overriden depending on the value of `link`.
    #[serde(rename = "dir-merge")]
    #[serde(default)]
    pub dir_merge: bool,

    /// `true` indicates that each tendril will have
    /// [`crate::TendrilMode::Link`], regardless of what the `dir_merge`
    /// setting is. `false` indicates that the `dir_merge` setting will be
    /// used.
    #[serde(default)]
    pub link: bool,

    /// A list of profiles to which this tendril belongs. If empty,
    /// this tendril is considered to be included in *all* profiles.
    #[serde(default)]
    #[serde(deserialize_with = "one_or_many_to_vec")]
    pub profiles: Vec<String>,
}

#[cfg(any(test, feature = "_test_utils"))]
impl From<RawTendril> for TendrilSet {
    fn from(raw: RawTendril) -> Self {
        let (dir_merge, link) = match raw.mode {
            TendrilMode::DirMerge => (true, false),
            TendrilMode::DirOverwrite => (false, false),
            TendrilMode::Link => (false, true),
        };

        TendrilSet {
            remotes: vec![raw.remote],
            dir_merge,
            link,
            profiles: raw.profiles,
        }
    }
}

#[cfg(any(test, feature = "_test_utils"))]
pub fn serialize_config(cfg: Config) -> String {
    let serde_cfg: SerdeConfig = cfg.into();
    serde_json::to_string(&serde_cfg).unwrap()
}

fn one_or_many_to_vec<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<String>, D::Error> {
    let one_or_many: OneOrMany<String> =
        de::Deserialize::deserialize(deserializer)?;
    Ok(one_or_many.into())
}

/// Parses the `tendrils.json` file in the given Tendrils repo and returns
/// the configuration within.
/// The tendril bundles are returned in the order they are defined in the file.
///
/// # Arguments
/// - `td_repo` - Path to the Tendrils folder.
pub(crate) fn get_config(
    td_repo: &UniPath,
) -> Result<Config, GetConfigError> {
    let config_file_path = td_repo.inner().join(".tendrils/tendrils.json");
    let config_file_contents = std::fs::read_to_string(config_file_path)?;
    let serde_config = parse_config(&config_file_contents)?;
    Ok(serde_config.into())
}

/// Parses the `~/.tendrils/global-config.json` file and returns the
/// configuration within. If the file doesn't exist, an empty configuration is
/// returned (i.e all fields set to `None`).
pub(crate) fn get_global_config() -> Result<GlobalConfig, GetConfigError> {
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
fn parse_config(
    json: &str
) -> Result<Config, serde_json::Error> {
    match serde_json::from_str::<SerdeConfig>(json) {
        Ok(raw) => Ok(raw.into()),
        Err(e) => Err(e)
    }
}

// Exposes the otherwise private function
#[cfg(test)]
pub fn parse_config_expose(
    json: &str,
) -> Result<Config, serde_json::Error> {
    parse_config(json)
}

/// # Arguments
/// - `json` - JSON object following the global-config.json schema
fn parse_global_config(
    json: &str
) -> Result<GlobalConfig, serde_json::Error> {
    serde_json::from_str::<GlobalConfig>(json)
}
