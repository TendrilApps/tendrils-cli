use crate::{ActionMode, TendrilBundle};
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
    tendrils: Vec<TendrilBundle>,
    filter: FilterSpec,
) -> Vec<TendrilBundle> {
    let mut filtered = match filter.mode {
        Some(v) => filter_by_mode(tendrils.to_vec(), v),
        None => tendrils.to_vec(),
    };

    filtered = filter_by_profiles(filtered, filter.profiles);
    filtered = filter_by_locals(filtered, filter.locals);
    filter_by_remotes(filtered, filter.remotes)
}

fn filter_by_mode(
    tendrils: Vec<TendrilBundle>,
    mode: ActionMode,
) -> Vec<TendrilBundle> {
    if mode == ActionMode::Out {
        return tendrils;
    }

    tendrils
        .into_iter()
        .filter(|t| t.link == (mode == ActionMode::Link))
        .collect()
}

fn filter_by_profiles(
    tendrils: Vec<TendrilBundle>,
    profiles: &[String],
) -> Vec<TendrilBundle> {
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
    tendrils: Vec<TendrilBundle>,
    locals: &[String],
) -> Vec<TendrilBundle> {
    if locals.is_empty() {
        return tendrils;
    }

    tendrils
        .into_iter()
        .filter(|t| locals.iter().any(|f| glob_match(f, &t.local)))
        .collect()
}

fn filter_by_remotes(
    mut tendrils: Vec<TendrilBundle>,
    remotes: &[String],
) -> Vec<TendrilBundle> {
    if remotes.is_empty() {
        return tendrils;
    }

    for t in tendrils.iter_mut() {
        let filtered_remotes_iter = t.remotes.iter().filter_map(|r| {
            if remotes.iter().any(|f| glob_match(f, r)) {
                Some(r.to_owned())
            }
            else {
                None
            }
        });
        t.remotes = filtered_remotes_iter.collect();
    }

    tendrils.into_iter().filter(|t| !t.remotes.is_empty()).collect()
}
