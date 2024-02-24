use crate::enums::OneOrMany;
use serde::{de, Deserialize, Deserializer, Serialize};

/// Represents a file system object that is controlled
/// by Tendrils.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tendril {
    pub group: String,
    pub name: String,

    #[serde(deserialize_with = "one_or_many_to_vec")]
    pub parents: Vec<String>,

    #[serde(rename = "dir-merge")]
    #[serde(default)]
    pub dir_merge: bool,

    #[serde(default)]
    pub link: bool,

    #[serde(default)]
    #[serde(deserialize_with = "one_or_many_to_vec")]
    pub profiles: Vec<String>
}

impl Tendril {
    pub fn id(&self) -> String {
        self.group.clone() + " - " + &self.name
    }

    #[cfg(test)]
    pub fn new(group: &str, name: &str) -> Tendril {
        Tendril {
            group: group.to_string(),
            name: name.to_string(),
            parents: vec![],
            dir_merge: false,
            link: false,
            profiles: vec![],
        }
    }
}

fn one_or_many_to_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where D: Deserializer<'de>
{
    let one_or_many: OneOrMany<String> = de::Deserialize::deserialize(deserializer)?;
    Ok(one_or_many.into())
}
