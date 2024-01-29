use serde::{Deserialize, Serialize};

/// Represents a file system object that is controlled
/// by Tendrils.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tendril {
    pub group: String,
    pub name: String,

    #[serde(rename = "parent-dirs-mac")]
    pub parent_dirs_mac: Vec<String>,

    #[serde(rename = "parent-dirs-windows")]
    pub parent_dirs_windows: Vec<String>,

    #[serde(rename = "folder-merge")]
    pub folder_merge: bool,

    pub link: bool,
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
            parent_dirs_mac: [].to_vec(),
            parent_dirs_windows: [].to_vec(),
            folder_merge: false,
            link: false,
        }
    }
}
