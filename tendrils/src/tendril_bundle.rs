use crate::enums::OneOrMany;
use serde::{de, Deserialize, Deserializer, Serialize};

/// Represents a bundle of file system objects that are controlled
/// by Tendrils.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TendrilBundle {
    /// The group by which this tendril will be sorted in
    /// the *Tendrils* folder.
    pub group: String,

    /// A list of file/folder names, each one belonging to each of the
    /// `parents`.
    #[serde(deserialize_with = "one_or_many_to_vec")]
    pub names: Vec<String>,

    /// A list of parent folders containing the files/folders in `names`.
    /// Each parent will be combined with each name to expand to individual
    /// tendrils.
    #[serde(deserialize_with = "one_or_many_to_vec")]
    pub parents: Vec<String>,

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

impl TendrilBundle {
    #[cfg(any(test, feature = "_test_utils"))]
    pub fn new(group: &str, names: Vec<&str>) -> TendrilBundle {
        TendrilBundle {
            group: String::from(group),
            names: names.into_iter().map(|n: &str| String::from(n)).collect(),
            parents: vec![],
            dir_merge: false,
            link: false,
            profiles: vec![],
        }
    }
}

fn one_or_many_to_vec<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<String>, D::Error> {
    let one_or_many: OneOrMany<String> =
        de::Deserialize::deserialize(deserializer)?;
    Ok(one_or_many.into())
}
