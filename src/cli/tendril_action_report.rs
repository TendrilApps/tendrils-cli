use crate::errors::{ResolveTendrilError, TendrilActionError};
use crate::resolved_tendril::ResolvedTendril;
use crate::tendril::Tendril;

pub struct TendrilActionReport<'a> {
    pub orig_tendril: &'a Tendril,
    pub resolve_results: Vec<Result<ResolvedTendril, ResolveTendrilError>>,
    pub action_results: Vec<Option<Result<(), TendrilActionError>>>,
}
