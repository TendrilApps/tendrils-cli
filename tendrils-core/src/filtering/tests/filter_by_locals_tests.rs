use crate::filtering::filter_by_locals;
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
fn empty_tendril_list_returns_empty(#[case] locals: &[String]) {
    let tendrils = vec![];

    let actual = filter_by_locals(tendrils, &locals);

    assert!(actual.is_empty())
}

#[apply(string_filter_match_tests)]
fn tendril_only_included_if_local_matches_any(
    #[case] filters: &[String],
    #[case] exp_matches: &[&str],
) {
    let t1 = RawTendril::new("v1");
    let t2 = RawTendril::new("v2");
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_locals(tendrils, &filters);

    let expected = match exp_matches {
        ["v1"] => vec![t1],
        ["v2"] => vec![t2],
        ["v1", "v2"] => vec![t1, t2],
        _ => panic!(),
    };

    assert_eq!(actual, expected);
}

#[apply(string_filter_non_match_tests)]
fn tendril_not_included_if_local_does_not_match_any(#[case] filters: &[String]) {
    let t1 = RawTendril::new("v1");
    let t2 = RawTendril::new("v2");
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_locals(tendrils, filters);

    assert!(actual.is_empty());
}

#[test]
fn duplicate_filter_only_returns_tendril_once() {
    let t1 = RawTendril::new("l1");
    let t2 = RawTendril::new("l2");
    let tendrils = vec![t1.clone(), t2.clone()];
    let filters = ["l1".to_string(), "l1".to_string(), "l1".to_string()];

    let actual = filter_by_locals(tendrils, &filters);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn duplicate_matching_tendrils_returns_all_instances() {
    let t1 = RawTendril::new("l1");
    let t2 = RawTendril::new("l2");
    let tendrils = vec![t1.clone(), t1.clone(), t1.clone(), t2.clone()];
    let filters = ["l1".to_string()];

    let actual = filter_by_locals(tendrils, &filters);

    assert_eq!(actual, vec![t1.clone(), t1.clone(), t1]);
}

#[apply(supported_weird_values)]
fn filter_supports_weird_filter_values(#[case] value: String) {
    let t1 = RawTendril::new(&value);
    let t2 = RawTendril::new("l2");
    let tendrils = vec![t1.clone(), t2.clone()];

    let filter = value.replace('\\', "\\\\");
    let actual = filter_by_locals(tendrils, &[filter]);

    assert_eq!(actual, vec![t1]);
}

#[apply(supported_asterisk_literals)]
fn filter_supports_asterisk_literals(
    #[case] value: String,
    #[case] filter: String,
) {
    let t1 = RawTendril::new(&value);
    let t2 = RawTendril::new("l2");
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_locals(tendrils, &[filter]);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn empty_filters_list_returns_all_tendrils() {
    let t1 = RawTendril::new("l1");
    let t2 = RawTendril::new("l2");
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_locals(tendrils.clone(), &[]);

    assert_eq!(actual, tendrils);
}
