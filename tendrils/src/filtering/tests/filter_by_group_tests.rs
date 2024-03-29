use crate::TendrilBundle;
use crate::filtering::filter_by_group;
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
fn empty_tendril_list_returns_empty(#[case] groups: &[String]) {
    let tendrils = vec![];

    let actual = filter_by_group(tendrils, &groups);

    assert!(actual.is_empty())
}

#[apply(string_filter_match_tests)]
fn tendril_only_included_if_any_group_matches(
    #[case] filters: &[String],
    #[case] exp_matches: &[&str],
) {
    let t1 = TendrilBundle::new("v1", vec!["misc.txt"]);
    let t2 = TendrilBundle::new("v2", vec!["misc.txt"]);
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_group(tendrils, &filters);

    let expected = match exp_matches {
        ["v1"] => vec![t1],
        ["v2"] => vec![t2],
        ["v1", "v2"] => vec![t1, t2],
        _ => panic!(),
    };

    assert_eq!(actual, expected);
}

#[apply(string_filter_non_match_tests)]
fn tendril_not_included_if_no_group_matches(
    #[case] filters: &[String]
) {
    let t1 = TendrilBundle::new("v1", vec!["misc.txt"]);
    let t2 = TendrilBundle::new("v2", vec!["misc.txt"]);
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_group(tendrils, filters);

    assert!(actual.is_empty());
}

#[test]
fn duplicate_filter_groups_only_returns_tendril_once() {
    let t1 = TendrilBundle::new("g1", vec!["misc.txt"]);
    let t2 = TendrilBundle::new("g2", vec!["misc.txt"]);
    let tendrils = vec![t1.clone(), t2.clone()];
    let filters = [
        "g1".to_string(),
        "g1".to_string(),
        "g1".to_string(),
    ];

    let actual = filter_by_group(tendrils, &filters);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn duplicate_tendrils_returns_all_instances() {
    let t1 = TendrilBundle::new("g1", vec!["misc.txt"]);
    let t2 = TendrilBundle::new("g2", vec!["misc.txt"]);
    let tendrils = vec![t1.clone(), t1.clone(), t1.clone(), t2.clone()];
    let filters = ["g1".to_string()];

    let actual = filter_by_group(tendrils, &filters);

    assert_eq!(actual, vec![t1.clone(), t1.clone(), t1]);
}

#[apply(supported_weird_values)]
fn filter_supports_weird_groups(
    #[case] group: String
) {
    let t1 = TendrilBundle::new(&group, vec!["misc.txt"]);
    let t2 = TendrilBundle::new("g2", vec!["misc.txt"]);
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_group(tendrils, &[group]);

    assert_eq!(actual, vec![t1]);
}

#[apply(supported_asterisk_literals)]
fn filter_supports_asterisk_literals(
    #[case] group: String,
    #[case] filter: String,
) {
    let t1 = TendrilBundle::new(&group, vec!["misc.txt"]);
    let t2 = TendrilBundle::new("g2", vec!["misc.txt"]);
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_group(tendrils, &[filter]);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn empty_filters_list_returns_all_tendrils() {
    let t1 = TendrilBundle::new("g1", vec!["misc.txt"]);
    let t2 = TendrilBundle::new("g2", vec!["misc.txt"]);
    let tendrils = vec![t1.clone(), t2.clone()];

    let actual = filter_by_group(tendrils.clone(), &[]);

    assert_eq!(actual, tendrils);
}
