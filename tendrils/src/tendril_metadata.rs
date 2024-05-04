use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Static metadata for a tendril.
pub struct TendrilMetadata {
    /// The full path to the remote. This shows the result after resolving
    /// all environment/other variables in the path.
    pub resolved_path: PathBuf,
}
