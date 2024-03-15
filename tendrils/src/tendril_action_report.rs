use crate::enums::{
    InvalidTendrilError,
    TendrilActionError,
    TendrilActionSuccess,
};
use crate::tendril::Tendril;
use std::path::PathBuf;

/// Contains the result of a single tendrils action.
#[derive(Debug)]
pub struct TendrilActionReport<'a> {
    /// The original tendril that this action was expanded from & performed
    /// on.
    pub orig_tendril: &'a Tendril,
    /// The name of the tendril that this action was performed on. If the
    /// `orig_tendril` contains multiple names, this indicates which was used.
    pub name: &'a str,
    /// The full path to the remote that was used for the action. This shows
    /// the result after resolving all environment/other variables in the path.
    pub resolved_path: Result<PathBuf, InvalidTendrilError>,
    /// Result of this individual action.
    pub action_result: Option<Result<TendrilActionSuccess, TendrilActionError>>,
}
