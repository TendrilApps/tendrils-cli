use crate::filtering::filter_by_remotes;
use crate::filtering::tests::filter_tendrils_tests::{
    string_filter_empty_tests,
    string_filter_match_tests,
    string_filter_non_match_tests,
    supported_asterisk_literals,
    supported_weird_values,
};
use crate::RawTendril;
use rstest::rstest;
use rstest_reuse::{self, apply};

#[apply(string_filter_empty_tests)]
fn empty_tendril_list_returns_empty(#[case] filters: &[String]) {
    let tendrils = vec![];

    let actual = filter_by_remotes(tendrils, &filters);

    assert!(actual.is_empty())
}

#[apply(string_filter_match_tests)]
fn tendril_remote_only_included_if_remote_matches_any(
    #[case] filters: &[String],
    #[case] exp_matches: &[&str],
) {
    let mut t1 = RawTendril::new("SomeLocal");
    let mut t2 = RawTendril::new("SomeLocal");
    t1.remote = "v1".to_string();
    t2.remote = "v2".to_string();
    
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_remotes(tendrils, &filters);

    let expected = match exp_matches {
        ["v1"] => vec![t1],
        ["v2"] => vec![t2],
        ["v1", "v2"] => vec![t1, t2],
        _ => panic!(),
    };

    assert_eq!(actual, expected);
}

#[apply(string_filter_non_match_tests)]
fn tendril_not_included_if_remote_does_not_match_any(#[case] filters: &[String]) {
    let mut t1 = RawTendril::new("SomeLocal");
    let mut t2 = RawTendril::new("SomeLocal");
    t1.remote = "v1".to_string();
    t2.remote = "v2".to_string();
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_remotes(tendrils, filters);

    assert!(actual.is_empty());
}

#[test]
fn duplicate_filter_only_returns_tendril_once() {
    let mut t1 = RawTendril::new("SomeLocal");
    let mut t2 = RawTendril::new("SomeLocal");
    t1.remote = "r1".to_string();
    t2.remote = "r2".to_string();
    let tendrils = vec![t1.clone(), t2.clone()];
    let filters = ["r1".to_string(), "r1".to_string(), "r1".to_string()];

    let actual = filter_by_remotes(tendrils, &filters);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn duplicate_matching_tendrils_returns_all_instances() {
    let mut t1 = RawTendril::new("SomeLocal");
    let mut t2 = RawTendril::new("SomeLocal");
    t1.remote = "r1".to_string();
    t2.remote = "r2".to_string();
    let tendrils = vec![t1.clone(), t1.clone(), t1.clone(), t2.clone()];
    let filters = ["r1".to_string()];

    let actual = filter_by_remotes(tendrils, &filters);

    assert_eq!(actual, vec![t1.clone(), t1.clone(), t1]);
}

#[apply(supported_weird_values)]
fn filter_supports_weird_filter_values(#[case] value: String) {
    let mut t1 = RawTendril::new("SomeLocal");
    let mut t2 = RawTendril::new("SomeLocal");
    t1.remote = value.clone();
    t2.remote = "r2".to_string();
    let tendrils = vec![t1.clone(), t2.clone()];

    let filter = value.replace('\\', "\\\\");
    let actual = filter_by_remotes(tendrils, &[filter]);

    assert_eq!(actual, vec![t1]);
}

#[apply(supported_asterisk_literals)]
fn filter_supports_asterisk_literals(
    #[case] value: String,
    #[case] filter: String,
) {
    let mut t1 = RawTendril::new("SomeLocal");
    let mut t2 = RawTendril::new("SomeLocal");
    t1.remote = value.clone();
    t2.remote = "r2".to_string();
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_remotes(tendrils, &[filter]);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn empty_filters_list_returns_all_tendrils() {
    let mut t1 = RawTendril::new("SomeLocal");
    let mut t2 = RawTendril::new("SomeLocal");
    t1.remote = "r1".to_string();
    t2.remote = "r2".to_string();
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_remotes(tendrils.clone(), &[]);

    assert_eq!(actual, tendrils);
}
