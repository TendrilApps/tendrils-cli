use crate::tendril_bundle::TendrilBundle;
use crate::enums::ActionMode;
use glob_match::glob_match;

#[cfg(test)]
mod tests;

/// Defines a series of filters that can be applied to a
/// list of tendrils.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FilterSpec<'a> {
    /// Matches only link style tendrils if the action mode is `Link`,
    /// otherwise it matches only the push/pull style tendrils. If
    /// `None`, all tendrils will match.
    pub mode: Option<ActionMode>,

    /// Matches only those tendril names that match any of the given names.
    /// Any tendril names that do not match are omitted, and any tendrils
    /// without any matching names are omitted entirely. Glob patterns
    /// are supported
    pub names: &'a [String],

    /// Matches only those tendrils that match any of the given profiles, and those
    /// that belong to all profiles (i.e. those that do not have any
    /// profiles defined). Glob patterns
    /// are supported
    pub profiles: &'a [String],
}

/// Filters a list of given tendrils according to the given [`FilterSpec`].
/// The filters are cumulative (i.e. the tendril must match all filters to
/// be included in the final result).
pub fn filter_tendrils(
    tendrils: Vec<TendrilBundle>, filter: FilterSpec
) -> Vec<TendrilBundle> {
    let mut filtered = match filter.mode {
        Some(v) => filter_by_mode(tendrils.to_vec(), v),
        None => tendrils.to_vec()
    };

    filtered = filter_by_profiles(filtered, filter.profiles);
    filter_by_names(filtered, filter.names)
}

fn filter_by_mode(tendrils: Vec<TendrilBundle>, mode: ActionMode) -> Vec<TendrilBundle> {
    tendrils.into_iter()
        .filter(|t| t.link == (mode == ActionMode::Link))
        .collect()
}

fn filter_by_profiles(tendrils: Vec<TendrilBundle>, profiles: &[String]) -> Vec<TendrilBundle> {
    if profiles.is_empty() {
        return tendrils;
    }

    tendrils.into_iter().filter(|t| -> bool {
        t.profiles.is_empty()
        || t.profiles.iter().any(|p| {
            profiles.iter().any(|f| glob_match(f, p))
        })
    }).collect()
}

fn filter_by_names(mut tendrils: Vec<TendrilBundle>, names: &[String]) -> Vec<TendrilBundle> {
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
