use crate::errors::{ResolveTendrilError, TendrilActionError};
use crate::tendril::Tendril;
use std::path::PathBuf;

pub struct TendrilActionReport<'a> {
    pub orig_tendril: &'a Tendril,
    pub resolved_paths: Vec<Result<PathBuf, ResolveTendrilError>>,
    pub action_results: Vec<Option<Result<(), TendrilActionError>>>,
}
