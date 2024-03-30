use crate::TendrilBundle;
use crate::filtering::filter_by_profiles;
use crate::filtering::tests::filter_tendrils_tests::{
    string_filter_empty_tests,
    string_filter_match_tests,
    string_filter_non_match_tests,
    supported_asterisk_literals,
    supported_weird_values,
};
use rstest::rstest;
use rstest_reuse::{self, apply};

#[apply(string_filter_empty_tests)]
fn empty_tendril_list_returns_empty(#[case] filters: &[String]) {
    let tendrils = vec![];

    let actual = filter_by_profiles(tendrils, filters);

    assert!(actual.is_empty())
}

#[apply(string_filter_empty_tests)]
#[case(&[])]
fn tendril_with_empty_profiles_list_included_in_all(
    #[case] filters: &[String]
) {
    let t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    assert!(t1.profiles.is_empty());
    let t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    assert!(t2.profiles.is_empty());
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_profiles(tendrils, filters);

    assert_eq!(actual, vec![t1, t2]);
}

#[apply(string_filter_match_tests)]
fn tendril_only_included_if_any_profile_matches(
    #[case] filters: &[String],
    #[case] exp_matches: &[&str],
) {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec!["v1".to_string(), "v2".to_string()];
    let tendrils = vec![t1.clone()];

    let actual = filter_by_profiles(tendrils, filters);

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

#[apply(string_filter_non_match_tests)]
fn tendril_not_included_if_not_empty_and_no_profile_matches(
    #[case] filters: &[String]
) {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec!["v1".to_string(), "v2".to_string()];
    let t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    assert!(t2.profiles.is_empty());
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_profiles(tendrils, filters);

    assert_eq!(actual, vec![t2]);
}

#[test]
fn duplicate_filter_profiles_only_returns_tendril_once() {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec!["P1".to_string()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    t2.profiles = vec!["P2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];
    let filters = [
        "P1".to_string(),
        "P1".to_string(),
        "P1".to_string(),
    ];

    let actual = filter_by_profiles(tendrils, &filters);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn duplicate_tendril_profiles_only_returns_tendril_once() {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec![
        "P1".to_string(),
        "P1".to_string(),
        "P1".to_string(),
    ];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    t2.profiles = vec!["P2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];
    let filters = ["P1".to_string()];

    let actual = filter_by_profiles(tendrils, &filters);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn duplicate_tendrils_returns_all_instances() {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec!["P1".to_string()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    t2.profiles = vec!["P2".to_string()];
    let tendrils = vec![t1.clone(), t1.clone(), t1.clone(), t2.clone()];
    let filters = ["P1".to_string()];

    let actual = filter_by_profiles(tendrils, &filters);

    assert_eq!(actual, vec![t1.clone(), t1.clone(), t1]);
}

#[apply(supported_weird_values)]
fn filter_supports_weird_profiles(
    #[case] profile: String
) {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec![profile.clone()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    t2.profiles = vec!["v2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];

    let filter = profile.replace('\\', "\\\\");
    let actual = filter_by_profiles(tendrils, &[filter]);

    assert_eq!(actual, vec![t1]);
}

#[apply(supported_asterisk_literals)]
fn filter_supports_asterisk_literals(
    #[case] profile: String,
    #[case] filter: String,
) {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec![profile];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    t2.profiles = vec!["P2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_profiles(tendrils, &[filter]);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn empty_filters_list_returns_all_tendrils() {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.profiles = vec!["P1".to_string()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    t2.profiles = vec![];
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_profiles(tendrils, &[]);

    assert_eq!(actual, vec![t1, t2]);
}
