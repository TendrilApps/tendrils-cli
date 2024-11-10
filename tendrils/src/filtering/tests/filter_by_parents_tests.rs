use crate::filtering::filter_by_remotes;
use crate::filtering::tests::filter_tendrils_tests::{
    string_filter_empty_tests,
    string_filter_match_tests,
    string_filter_non_match_tests,
    supported_asterisk_literals,
    supported_weird_values,
};
use crate::TendrilBundle;
use rstest::rstest;
use rstest_reuse::{self, apply};

#[apply(string_filter_empty_tests)]
fn empty_tendril_list_returns_empty(#[case] filters: &[String]) {
    let tendrils = vec![];

    let actual = filter_by_remotes(tendrils, &filters);

    assert!(actual.is_empty())
}

#[apply(string_filter_empty_tests)]
fn tendril_with_empty_remotes_list_not_included(#[case] filters: &[String]) {
    let mut t1 = TendrilBundle::new("SomeLocal");
    t1.remotes = vec![];
    let mut t2 = TendrilBundle::new("SomeLocal");
    t2.remotes = vec![];
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_remotes(tendrils, filters);

    assert!(actual.is_empty());
}

#[apply(string_filter_match_tests)]
fn tendril_remote_only_included_if_it_matches_and_non_matching_remotes_are_omitted(
    #[case] filters: &[String],
    #[case] exp_matches: &[&str],
) {
    let mut t1 = TendrilBundle::new("SomeLocal");
    t1.remotes = vec!["v1".to_string(), "v2".to_string()];
    let tendrils = vec![t1.clone()];

    let actual = filter_by_remotes(tendrils, &filters);

    // Check that ONLY the expected matches are included in the
    // returned remotes and that non-matching remotes were omitted
    let mut expected = t1.clone();
    expected.remotes = exp_matches.into_iter().map(|v| v.to_string()).collect();
    assert_eq!(actual, vec![expected]);
}

#[test]
fn remote_included_if_any_pattern_matches() {
    let mut t1 = TendrilBundle::new("SomeLocal");
    t1.remotes = vec!["r/1".to_string(), "r/2".to_string()];
    let tendrils = vec![t1.clone()];
    let filters = vec![
        "I don't match".to_string(),
        "me neither".to_string(),
        "r/1".to_string(),
    ];

    let actual = filter_by_remotes(tendrils, &filters);

    let mut expected = t1.clone();
    expected.remotes = vec!["r/1".to_string()];
    assert_eq!(actual, vec![expected]);
}

#[apply(string_filter_non_match_tests)]
fn tendril_not_included_if_no_remote_matches(#[case] filters: &[String]) {
    let mut t1 = TendrilBundle::new("SomeLocal");
    t1.remotes = vec!["v1".to_string(), "v2".to_string()];
    let mut t2 = TendrilBundle::new("SomeLocal");
    t2.remotes = vec![];
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_remotes(tendrils, filters);

    assert!(actual.is_empty());
}

#[test]
fn duplicate_filter_remotes_only_returns_tendril_once() {
    let mut t1 = TendrilBundle::new("SomeLocal");
    t1.remotes = vec!["r/1".to_string()];
    let mut t2 = TendrilBundle::new("SomeLocal");
    t2.remotes = vec!["r/2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];
    let filters = ["r/1".to_string(), "r/1".to_string(), "r/1".to_string()];

    let actual = filter_by_remotes(tendrils, &filters);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn duplicate_tendril_remotes_only_returns_tendril_once() {
    let mut t1 = TendrilBundle::new("SomeLocal");
    t1.remotes = vec!["r/1".to_string(), "r/1".to_string(), "r/1".to_string()];
    let mut t2 = TendrilBundle::new("SomeLocal");
    t2.remotes = vec!["r/2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];
    let filters = ["r/1".to_string()];

    let actual = filter_by_remotes(tendrils, &filters);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn duplicate_tendrils_returns_all_instances() {
    let mut t1 = TendrilBundle::new("SomeLocal");
    t1.remotes = vec!["r/1".to_string()];
    let mut t2 = TendrilBundle::new("SomeLocal");
    t2.remotes = vec!["r/2".to_string()];
    let tendrils = vec![t1.clone(), t1.clone(), t1.clone(), t2.clone()];
    let filters = ["r/1".to_string()];

    let actual = filter_by_remotes(tendrils, &filters);

    assert_eq!(actual, vec![t1.clone(), t1.clone(), t1]);
}

#[apply(supported_weird_values)]
fn filter_supports_weird_remotes(#[case] remote: String) {
    let mut t1 = TendrilBundle::new("SomeLocal");
    t1.remotes = vec![remote.clone()];
    let mut t2 = TendrilBundle::new("SomeLocal");
    t2.remotes = vec!["r/2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];

    let filter = remote.replace('\\', "\\\\");
    let actual = filter_by_remotes(tendrils, &[filter]);

    assert_eq!(actual, vec![t1]);
}

#[apply(supported_asterisk_literals)]
fn filter_supports_asterisk_literals(
    #[case] remote: String,
    #[case] filter: String,
) {
    let mut t1 = TendrilBundle::new("SomeLocal");
    t1.remotes = vec![remote];
    let mut t2 = TendrilBundle::new("SomeLocal");
    t2.remotes = vec!["r/2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_remotes(tendrils, &[filter]);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn empty_filters_list_returns_all_tendrils() {
    let mut t1 = TendrilBundle::new("SomeLocal");
    t1.remotes = vec!["r/1".to_string()];
    let mut t2 = TendrilBundle::new("SomeLocal");
    t2.remotes = vec!["r/2".to_string()];
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_remotes(tendrils.clone(), &[]);

    assert_eq!(actual, tendrils);
}
