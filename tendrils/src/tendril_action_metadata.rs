use crate::TendrilMetadata;
use crate::enums::{
    TendrilActionError,
    TendrilActionSuccess,
};

/// Contains the metadata from a single tendrils action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TendrilActionMetadata {
    pub md: TendrilMetadata,
    /// Result of this individual action.
    pub action_result: Result<TendrilActionSuccess, TendrilActionError>,
}
