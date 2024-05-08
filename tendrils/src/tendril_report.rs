use crate::{
    InvalidTendrilError,
    TendrilActionMetadata,
    TendrilBundle,
};

/// Contains the result of a single tendrils action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TendrilActionReport<'a> {
    /// The original tendril bundle that this action was expanded from & performed
    /// on.
    pub orig_tendril: &'a TendrilBundle,
    /// The name of the tendril that this action was performed on. If the
    /// `orig_tendril` contains multiple names, this indicates which was used.
    pub name: &'a str,
    /// Result containing the [metadata](`TendrilActionMetadata`) from the
    /// action, provided the `orig_tendril` was valid.
    /// Otherwise, it contains the [`InvalidTendrilError`].
    pub metadata: Result<TendrilActionMetadata, InvalidTendrilError>,
}
