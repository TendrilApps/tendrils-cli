use crate::FsoType;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Static metadata for a tendril.
pub struct TendrilMetadata {
    /// The type of the file system object in the Tendrils folder.
    /// `None` if it does not exist.
    pub local_type: Option<FsoType>,

    /// The type of the file system object at its remote location on
    /// the device.
    /// `None` if it does not exist.
    pub remote_type: Option<FsoType>,

    /// The full path to the remote. This shows the result after resolving
    /// all environment/other variables in the path.
    pub resolved_path: PathBuf,
}
