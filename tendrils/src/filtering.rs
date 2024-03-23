use crate::tendril_bundle::TendrilBundle;
use crate::enums::ActionMode;

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

    /// Matches only those tendrils that match any of the given profiles, and those
    /// that belong to all profiles (i.e. those that do not have any
    /// profiles defined).
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

    filtered
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

    tendrils.into_iter()
        .filter(|t| -> bool {
            t.profiles.is_empty()
            || profiles.iter().any(|p| t.profiles.contains(p))
        })
        .collect()
}
