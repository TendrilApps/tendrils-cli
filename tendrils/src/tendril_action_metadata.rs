use crate::enums::{
    TendrilActionError,
    TendrilActionSuccess,
};
use std::path::PathBuf;

/// Contains the metadata from a single tendrils action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TendrilActionMetadata {
    /// The full path to the remote that was used for the action. This shows
    /// the result after resolving all environment/other variables in the path.
    pub resolved_path: PathBuf,
    /// Result of this individual action.
    pub action_result: Result<TendrilActionSuccess, TendrilActionError>,
}
