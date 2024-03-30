use crate::TendrilBundle;
use crate::filtering::filter_by_parents;
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

    let actual = filter_by_parents(tendrils, &filters);

    assert!(actual.is_empty())
}

#[apply(string_filter_empty_tests)]
fn tendril_with_empty_parents_list_not_included(
    #[case] filters: &[String]
) {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.parents = vec![];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t2.parents = vec![];
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_parents(tendrils, filters);

    assert!(actual.is_empty());
}

#[apply(string_filter_match_tests)]
fn tendril_parent_only_included_if_it_matches_and_non_matching_parents_are_omitted(
    #[case] filters: &[String],
    #[case] exp_matches: &[&str],
) {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.parents = vec!["v1".to_string(), "v2".to_string()];
    let tendrils = vec![t1.clone()];

    let actual = filter_by_parents(tendrils, &filters);

    // Check that ONLY the expected matches are included in the
    // returned parents and that non-matching parents were omitted
    let mut expected = t1.clone();
    expected.parents = exp_matches.into_iter().map(|v| v.to_string()).collect();
    assert_eq!(actual, vec![expected]);
}

#[test]
fn parent_included_if_any_pattern_matches() {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.parents = vec!["p/1".to_string(), "p/2".to_string()];
    let tendrils = vec![t1.clone()];
    let filters = vec![
        "I don't match".to_string(), "me neither".to_string(), "p/1".to_string()
    ];

    let actual = filter_by_parents(tendrils, &filters);

    let mut expected = t1.clone();
    expected.parents = vec!["p/1".to_string()];
    assert_eq!(actual, vec![expected]);
}

#[apply(string_filter_non_match_tests)]
fn tendril_not_included_if_no_parent_matches(
    #[case] filters: &[String]
) {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.parents = vec!["v1".to_string(), "v2".to_string()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t2.parents = vec![];
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_parents(tendrils, filters);

    assert!(actual.is_empty());
}

#[test]
fn duplicate_filter_parents_only_returns_tendril_once() {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.parents = vec!["p/1".to_string()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t2.parents = vec!["p/2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];
    let filters = [
        "p/1".to_string(),
        "p/1".to_string(),
        "p/1".to_string(),
    ];

    let actual = filter_by_parents(tendrils, &filters);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn duplicate_tendril_parents_only_returns_tendril_once() {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.parents = vec![
        "p/1".to_string(),
        "p/1".to_string(),
        "p/1".to_string(),
    ];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t2.parents = vec!["p/2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];
    let filters = ["p/1".to_string()];

    let actual = filter_by_parents(tendrils, &filters);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn duplicate_tendrils_returns_all_instances() {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.parents = vec!["p/1".to_string()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t2.parents = vec!["p/2".to_string()];
    let tendrils = vec![t1.clone(), t1.clone(), t1.clone(), t2.clone()];
    let filters = ["p/1".to_string()];

    let actual = filter_by_parents(tendrils, &filters);

    assert_eq!(actual, vec![t1.clone(), t1.clone(), t1]);
}

#[apply(supported_weird_values)]
fn filter_supports_weird_parents(
    #[case] parent: String
) {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.parents  = vec![parent.clone()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t2.parents = vec!["p/2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];

    let filter = parent.replace('\\', "\\\\");
    let actual = filter_by_parents(tendrils, &[filter]);

    assert_eq!(actual, vec![t1]);
}

#[apply(supported_asterisk_literals)]
fn filter_supports_asterisk_literals(
    #[case] parent: String,
    #[case] filter: String,
) {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.parents  = vec![parent];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t2.parents = vec!["p/2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_parents(tendrils, &[filter]);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn empty_filters_list_returns_all_tendrils() {
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t1.parents = vec!["p/1".to_string()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    t2.parents = vec!["p/2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_parents(tendrils.clone(), &[]);

    assert_eq!(actual, tendrils);
}
