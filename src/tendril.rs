use serde::{ Serialize, Deserialize };

/// Represents a file system object that is controlled
/// by Tendrils.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tendril {
    pub app: String,
    pub name: String,

    #[serde(rename = "parent-dirs-mac")]
    pub parent_dirs_mac: Vec<String>,

    #[serde(rename = "parent-dirs-windows")]
    pub parent_dirs_windows: Vec<String>,

    #[serde(rename = "folder-merge")]
    pub folder_merge: bool,
}

impl Tendril {
    pub fn id(&self) -> String {
        self.app.clone() + " - " + &self.name
    }
}
