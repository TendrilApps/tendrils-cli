use crate::TendrilBundle;
use crate::filtering::filter_by_names;
use crate::filtering::tests::filter_tendrils_tests::{
    string_filter_empty_tests,
    string_filter_exact_match_tests,
    string_filter_non_exact_match_tests,
    supported_weird_values,
};
use rstest::rstest;
use rstest_reuse::{self, apply};

#[apply(string_filter_empty_tests)]
fn empty_tendril_list_returns_empty(#[case] names: &[String]) {
    let tendrils = vec![];

    let actual = filter_by_names(tendrils, &names);

    assert!(actual.is_empty())
}

#[apply(string_filter_exact_match_tests)]
fn tendril_only_included_if_any_name_matches_exactly_and_non_matching_names_are_omitted(
    #[case] names: &[String],
    #[case] exp_matches: &[&str],
) {
    let t1 = TendrilBundle::new("SomeApp", vec!["v1", "v2"]);
    let t2 = TendrilBundle::new("SomeApp", vec!["v4"]);
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_names(tendrils, &names);

    // Check that ONLY the expected matches are included in the
    // returned names and that non-matching names were omitted
    let mut expected = t1.clone();
    expected.names = exp_matches.into_iter().map(|v| v.to_string()).collect();
    assert_eq!(actual, vec![expected]);
}

#[apply(string_filter_non_exact_match_tests)]
fn tendril_not_included_if_no_name_matches_exactly(
    #[case] names: &[String]
) {
    let t1 = TendrilBundle::new("SomeApp", vec!["v1", "v2"]);
    let t2 = TendrilBundle::new("SomeApp", vec![]);
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_names(tendrils, names);

    assert!(actual.is_empty());
}

#[test]
fn duplicate_filter_names_only_returns_tendril_once() {
    let t1 = TendrilBundle::new("SomeApp", vec!["n1"]);
    let t2 = TendrilBundle::new("SomeApp", vec!["n2"]);
    let tendrils = vec![t1.clone(), t2.clone()];
    let given_filters = [
        "n1".to_string(),
        "n1".to_string(),
        "n1".to_string(),
    ];

    let actual = filter_by_names(tendrils, &given_filters);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn duplicate_tendrils_returns_all_instances() {
    let t1 = TendrilBundle::new("SomeApp", vec!["n1"]);
    let t2 = TendrilBundle::new("SomeApp", vec!["n2"]);
    let tendrils = vec![t1.clone(), t1.clone(), t1.clone(), t2.clone()];
    let given_filters = ["n1".to_string()];

    let actual = filter_by_names(tendrils, &given_filters);

    assert_eq!(actual, vec![t1.clone(), t1.clone(), t1]);
}

#[apply(supported_weird_values)]
fn filter_supports_weird_names(
    #[case] name: String
) {
    let t1 = TendrilBundle::new("SomeApp", vec![&name]);
    let t2 = TendrilBundle::new("SomeApp", vec!["n2"]);
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_names(tendrils, &[name]);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn empty_filters_list_returns_all_tendrils() {
    let t1 = TendrilBundle::new("SomeApp", vec!["n1"]);
    let t2 = TendrilBundle::new("SomeApp", vec!["n2"]);
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_names(tendrils.clone(), &[]);

    assert_eq!(actual, tendrils);
}
