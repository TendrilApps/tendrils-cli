use crate::TendrilBundle;
use crate::filtering::filter_by_profiles;
use crate::filtering::tests::filter_tendrils_tests::{
    string_filter_empty_tests,
    string_filter_exact_match_tests,
    string_filter_non_exact_match_tests,
    supported_weird_values,
};
use rstest::rstest;
use rstest_reuse::{self, apply};

#[apply(string_filter_empty_tests)]
fn empty_tendril_list_returns_empty(#[case] profiles: &[String]) {
    let tendrils = vec![];

    let actual = filter_by_profiles(tendrils, profiles);

    assert!(actual.is_empty())
}

#[apply(string_filter_empty_tests)]
fn tendril_with_empty_profiles_included_in_all(
    #[case] profiles: &[String]
) {
    let t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    assert!(t1.profiles.is_empty());
    let t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    assert!(t2.profiles.is_empty());
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_profiles(tendrils, profiles);

    assert_eq!(actual, vec![t1, t2]);
}

#[apply(string_filter_exact_match_tests)]
fn tendril_only_included_if_any_profile_matches_exactly(
    #[case] profiles: &[String],
    #[case] exp_matches: &[&str],
) {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec!["v1".to_string(), "v2".to_string()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    t2.profiles = vec!["v4".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_profiles(tendrils, profiles);

    assert_eq!(actual, vec![t1.clone()]);
    // Check that at least one of the expected profile
    // matches is included. Non-matching profiles are still
    // included in the list (unlike name filtering).
    assert!(
        t1.profiles.iter().any(|p| {
            exp_matches.contains(&p.as_str())
        })
    );
}

#[apply(string_filter_non_exact_match_tests)]
fn tendril_not_included_if_not_empty_and_no_profile_matches_exactly(
    #[case] profiles: &[String]
) {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec!["v1".to_string(), "v2".to_string()];
    let t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    assert!(t2.profiles.is_empty());
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_profiles(tendrils, profiles);

    assert_eq!(actual, vec![t2]);
}

#[test]
fn duplicate_filter_profiles_only_returns_tendril_once() {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec!["p1".to_string()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    t2.profiles = vec!["p2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];
    let given_filters = [
        "p1".to_string(),
        "p1".to_string(),
        "p1".to_string(),
    ];

    let actual = filter_by_profiles(tendrils, &given_filters);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn duplicate_tendril_profiles_only_returns_tendril_once() {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec![
        "p1".to_string(),
        "p1".to_string(),
        "p1".to_string(),
    ];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    t2.profiles = vec!["p2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];
    let given_filters = ["p1".to_string()];

    let actual = filter_by_profiles(tendrils, &given_filters);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn duplicate_tendrils_returns_all_instances() {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec!["p1".to_string()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    t2.profiles = vec!["p2".to_string()];
    let tendrils = vec![t1.clone(), t1.clone(), t1.clone(), t2.clone()];
    let given_filters = ["p1".to_string()];

    let actual = filter_by_profiles(tendrils, &given_filters);

    assert_eq!(actual, vec![t1.clone(), t1.clone(), t1]);
}

#[apply(supported_weird_values)]
fn filter_supports_weird_profiles(
    #[case] profile: String
) {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec![profile.clone()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    t2.profiles = vec!["v1".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_profiles(tendrils, &[profile]);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn empty_filters_list_returns_all_tendrils() {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec!["p1".to_string()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    t2.profiles = vec![];
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_profiles(tendrils, &[]);

    assert_eq!(actual, vec![t1, t2]);
}
