use crate::{ActionMode, RawTendril, TendrilMode};
use glob_match::glob_match;

#[cfg(test)]
mod tests;

/// Defines a series of filters that can be applied to a
/// list of tendrils.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilterSpec<'a> {
    /// Matches only link style tendrils if the action mode is `Link`,
    /// otherwise it matches only the push/pull style tendrils. If
    /// `None`, or [`ActionMode::Out`] all tendrils will match.
    pub mode: Option<ActionMode>,

    /// Matches only those tendrils whose local matches any of the given
    /// locals. Glob patterns are supported.
    pub locals: &'a [String],

    /// Matches only those tendril remotes that match any of the given remotes.
    /// Any tendril remotes that do not match are omitted, and any tendrils
    /// without any matching remotes are omitted entirely. Glob patterns
    /// are supported.
    pub remotes: &'a [String],

    /// Matches only those tendrils that match any of the given profiles, and
    /// those that belong to all profiles (i.e. those that do not have any
    /// profiles defined). Glob patterns
    /// are supported.
    pub profiles: &'a [String],
}

impl<'a> FilterSpec<'a> {
    pub fn new() -> FilterSpec<'a> {
        FilterSpec {
            mode: None,
            locals: &[],
            remotes: &[],
            profiles: &[],
        }
    }
}

/// Filters a list of given tendrils according to the given [`FilterSpec`].
/// The filters are cumulative (i.e. the tendril must match all filters to
/// be included in the final result).
pub(crate) fn filter_tendrils(
    tendrils: Vec<RawTendril>,
    filter: FilterSpec,
) -> Vec<RawTendril> {
    let mut filtered = match filter.mode {
        Some(v) => filter_by_mode(tendrils.to_vec(), v),
        None => tendrils.to_vec(),
    };

    filtered = filter_by_profiles(filtered, filter.profiles);
    filtered = filter_by_locals(filtered, filter.locals);
    filter_by_remotes(filtered, filter.remotes)
}

fn filter_by_mode(
    tendrils: Vec<RawTendril>,
    mode: ActionMode,
) -> Vec<RawTendril> {
    if mode == ActionMode::Out {
        return tendrils;
    }

    tendrils
        .into_iter()
        .filter(|t| match (&t.mode, &mode) {
            (TendrilMode::Link, ActionMode::Link) => true,
            (TendrilMode::Link, _) => false,
            (_, ActionMode::Link) => false,
            (_, _) => true,
        })
        .collect()
}

fn filter_by_profiles(
    tendrils: Vec<RawTendril>,
    profiles: &[String],
) -> Vec<RawTendril> {
    if profiles.is_empty() {
        return tendrils;
    }

    tendrils
        .into_iter()
        .filter(|t| -> bool {
            t.profiles.is_empty()
                || t.profiles
                    .iter()
                    .any(|p| profiles.iter().any(|f| glob_match(f, p)))
        })
        .collect()
}

fn filter_by_locals(
    tendrils: Vec<RawTendril>,
    locals: &[String],
) -> Vec<RawTendril> {
    if locals.is_empty() {
        return tendrils;
    }

    tendrils
        .into_iter()
        .filter(|t| locals.iter().any(|f| glob_match(f, &t.local)))
        .collect()
}

fn filter_by_remotes(
    tendrils: Vec<RawTendril>,
    remotes: &[String],
) -> Vec<RawTendril> {
    if remotes.is_empty() {
        return tendrils;
    }

    tendrils
        .into_iter()
        .filter(|t| remotes.iter().any(|f| glob_match(f, &t.remote)))
        .collect()
}
