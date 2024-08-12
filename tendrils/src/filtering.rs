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

    /// Matches only those tendrils whose group matches any of the given
    /// groups. Glob patterns are supported.
    pub groups: &'a [String],

    /// Matches only those tendril names that match any of the given names.
    /// Any tendril names that do not match are omitted, and any tendrils
    /// without any matching names are omitted entirely. Glob patterns
    /// are supported.
    pub names: &'a [String],

    /// Matches only those tendril parents that match any of the given parents.
    /// Any tendril parents that do not match are omitted, and any tendrils
    /// without any matching parents are omitted entirely. Glob patterns
    /// are supported.
    pub parents: &'a [String],

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
            groups: &[],
            names: &[],
            parents: &[],
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
    filtered = filter_by_group(filtered, filter.groups);
    filtered = filter_by_names(filtered, filter.names);
    filter_by_parents(filtered, filter.parents)
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

fn filter_by_group(
    tendrils: Vec<TendrilBundle>,
    groups: &[String],
) -> Vec<TendrilBundle> {
    if groups.is_empty() {
        return tendrils;
    }

    tendrils
        .into_iter()
        .filter(|t| groups.iter().any(|f| glob_match(f, &t.group)))
        .collect()
}

fn filter_by_names(
    mut tendrils: Vec<TendrilBundle>,
    names: &[String],
) -> Vec<TendrilBundle> {
    if names.is_empty() {
        return tendrils;
    }

    for t in tendrils.iter_mut() {
        let filtered_names_iter = t.names.iter().filter_map(|n| {
            if names.iter().any(|f| glob_match(f, n)) {
                Some(n.to_owned())
            }
            else {
                None
            }
        });
        t.names = filtered_names_iter.collect();
    }

    tendrils.into_iter().filter(|t| !t.names.is_empty()).collect()
}

fn filter_by_parents(
    mut tendrils: Vec<TendrilBundle>,
    parents: &[String],
) -> Vec<TendrilBundle> {
    if parents.is_empty() {
        return tendrils;
    }

    for t in tendrils.iter_mut() {
        let filtered_parents_iter = t.parents.iter().filter_map(|p| {
            if parents.iter().any(|f| glob_match(f, p)) {
                Some(p.to_owned())
            }
            else {
                None
            }
        });
        t.parents = filtered_parents_iter.collect();
    }

    tendrils.into_iter().filter(|t| !t.parents.is_empty()).collect()
}
