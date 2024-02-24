use crate::enums::{
    ResolveTendrilError,
    TendrilActionError,
    TendrilActionSuccess,
};
use crate::tendril::Tendril;
use std::path::PathBuf;

#[derive(Debug)]
pub struct TendrilActionReport<'a> {
    pub orig_tendril: &'a Tendril,
    pub resolved_path: Result<PathBuf, ResolveTendrilError>,
    pub action_result: Option<Result<TendrilActionSuccess, TendrilActionError>>,
}
