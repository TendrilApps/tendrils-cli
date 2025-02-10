use crate::{ActionMode, RawTendril, TendrilMode};
use crate::config::LazyCachedGlobalConfig;
use crate::filtering::{filter_tendrils, FilterSpec};
use rstest_reuse::{self, template};

#[template]
#[rstest]
#[case(&["".to_string()])]
#[case(&["*".to_string()])]
#[case(&["**".to_string()])]
#[case(&["v1".to_string()])]
#[case(&["*v1*".to_string()])]
#[case(&["**v1**".to_string()])]
#[case(&["v1".to_string(), "v2".to_string()])]
fn string_filter_empty_tests(#[case] filters: &[String]) {}

/// Expected matches based on a field under test with values ["v1", "v2"]
#[template]
#[rstest]
#[case(&["v1".to_string()], &["v1"])]
#[case(&["v2".to_string()], &["v2"])]
#[case(&["".to_string(), "v1".to_string()], &["v1"])]
#[case(&["v1".to_string(), "v2".to_string()], &["v1", "v2"])]
#[case(&["v1".to_string(), "v3".to_string()], &["v1"])]
#[case(&["v2".to_string(), "v3".to_string()], &["v2"])]
#[case(&["v1".to_string(), "v2".to_string(), "v3".to_string()], &["v1", "v2"])]
#[case(&["*".to_string()], &["v1", "v2"])]
#[case(&["**".to_string()], &["v1", "v2"])]
#[case(&["v?".to_string()], &["v1", "v2"])]
#[case(&["*?".to_string()], &["v1", "v2"])]
#[case(&["??".to_string()], &["v1", "v2"])]
#[case(&["*1".to_string()], &["v1"])]
#[case(&["?1".to_string()], &["v1"])]
#[case(&["!v1".to_string()], &["v2"])]
#[case(&["v[12]".to_string()], &["v1", "v2"])]
#[case(&["v{!1,2}".to_string()], &["v2"])]
fn string_filter_match_tests(
    #[case] filters: &[String],
    #[case] exp_matches: &[&str],
) {
}

/// Expected to not match based on a field under test with values ["v1", "v2"]
#[template]
#[rstest]
#[case(&["".to_string()])]
#[case(&["V1".to_string()])]
#[case(&["V2".to_string()])]
#[case(&["v3".to_string()])]
#[case(&["v1 ".to_string()])]
#[case(&[" v1".to_string()])]
#[case(&[" v1".to_string()])]
#[case(&["v1Leading".to_string()])]
#[case(&["Trailingv1".to_string()])]
#[case(&["V1".to_string(), "V2".to_string(), "v3".to_string()])]
fn string_filter_non_match_tests(#[case] filters: &[String]) {}

#[template]
#[rstest]
#[case("")]
#[case(" ")]
#[case("\n")]
#[case("\t")]
#[case("\r")]
#[case("Unix/Style/Paths")]
#[case("Windows\\Style\\Paths")]
fn supported_weird_values(#[case] filter: &str) {}

#[template]
#[rstest]
#[case("*", "\\*")]
#[case("**", "\\*\\*")]
fn supported_asterisk_literals(#[case] value: &str, #[case] filter: &str) {}

fn samples() -> Vec<RawTendril> {
    let mut t0 = RawTendril::new("l0");
    t0.mode = TendrilMode::DirOverwrite;
    t0.remote = "r0".to_string();
    t0.profiles = vec![];
    let mut t1 = RawTendril::new("l1");
    t0.mode = TendrilMode::DirOverwrite;
    t1.remote = "r1".to_string();
    t1.profiles = vec!["p1".to_string()];
    let mut t2 = RawTendril::new("l2");
    t0.mode = TendrilMode::Link;
    t2.remote = "r2".to_string();
    t2.profiles = vec!["p2".to_string()];
    let mut t3 = RawTendril::new("l3");
    t0.mode = TendrilMode::DirOverwrite;
    t3.remote = "r3".to_string();
    t3.profiles = vec!["p3".to_string()];
    let mut t4 = RawTendril::new("l4");
    t0.mode = TendrilMode::DirOverwrite;
    t4.remote = "r4".to_string();
    t4.profiles = vec!["p4".to_string()];
    let mut t5 = RawTendril::new("l5");
    t0.mode = TendrilMode::DirOverwrite;
    t5.remote = "r5".to_string();
    t5.profiles = vec!["p5".to_string()];

    vec![t0, t1, t2, t3, t4, t5]
}

#[test]
fn empty_tendril_list_returns_empty() {
    let tendrils = vec![];
    let filter = FilterSpec {
        mode: None,
        locals: vec![],
        remotes: vec![],
        profiles: None,
    };
    let mut cfg = LazyCachedGlobalConfig::mock_w_parse_err();

    let actual = filter_tendrils(tendrils, filter, &mut cfg);

    assert!(actual.is_empty())
}

#[test]
fn mode_filter_is_none_does_not_filter_by_mode() {
    let tendrils = samples();
    let filter = FilterSpec {
        mode: None,
        locals: vec!["l0".to_string(), "l1".to_string(), "l2".to_string()],
        remotes: vec!["r0".to_string(), "r1".to_string(), "r2".to_string()],
        profiles: Some(vec!["p1".to_string(), "p2".to_string()]),
    };
    let mut cfg = LazyCachedGlobalConfig::mock_w_parse_err();

    let actual = filter_tendrils(tendrils.clone(), filter, &mut cfg);

    assert_eq!(actual, vec![
        tendrils[0].clone(), // Copy type
        tendrils[1].clone(), // Copy type
        tendrils[2].clone(), // Link type
    ]);
}

#[test]
fn locals_filter_is_empty_does_not_filter_by_locals() {
    let tendrils = samples();
    let filter = FilterSpec {
        mode: Some(ActionMode::Pull),
        locals: vec![],
        remotes: vec!["r0".to_string(), "r1".to_string(), "r3".to_string()],
        profiles: Some(vec!["p1".to_string(), "p3".to_string()]),
    };
    let mut cfg = LazyCachedGlobalConfig::mock_w_parse_err();

    let actual = filter_tendrils(tendrils.clone(), filter, &mut cfg);

    assert_eq!(actual, vec![
        tendrils[0].clone(),
        tendrils[1].clone(),
        tendrils[3].clone(),
    ]);
}

#[test]
fn parent_filter_is_empty_does_not_filter_by_parent() {
    let tendrils = samples();
    let filter = FilterSpec {
        mode: Some(ActionMode::Pull),
        locals: vec!["l0".to_string(), "l1".to_string(), "l3".to_string()],
        remotes: vec![],
        profiles: Some(vec!["p1".to_string(), "p3".to_string()]),
    };
    let mut cfg = LazyCachedGlobalConfig::mock_w_parse_err();

    let actual = filter_tendrils(tendrils.clone(), filter, &mut cfg);

    assert_eq!(actual, vec![
        tendrils[0].clone(),
        tendrils[1].clone(),
        tendrils[3].clone(),
    ]);
}

#[test]
fn profile_filter_is_empty_does_not_filter_by_profile() {
    let tendrils = samples();
    let filter = FilterSpec {
        mode: Some(ActionMode::Pull),
        locals: vec!["l0".to_string(), "l1".to_string(), "l3".to_string()],
        remotes: vec!["r0".to_string(), "r1".to_string(), "r3".to_string()],
        profiles: Some(vec![]),
    };
    let mut cfg = LazyCachedGlobalConfig::mock_w_parse_err();

    let actual = filter_tendrils(tendrils.clone(), filter, &mut cfg);

    assert_eq!(actual, vec![
        tendrils[0].clone(),
        tendrils[1].clone(),
        tendrils[3].clone(),
    ]);
}

// Requires n+1 sample tendrils for n filter criteria as each
// filter is designed to eliminate one tendril
#[test]
fn all_filters_are_cumulative() {
    let tendrils = samples();
    let filter = FilterSpec {
        mode: Some(ActionMode::Pull), // Eliminates t2
        locals: vec![
            "l2".to_string(),
            "l3".to_string(),
            "l4".to_string(),
            "l5".to_string(),
        ], // Eliminates t0 & t1
        remotes: vec![
            "r0".to_string(),
            "r1".to_string(),
            "r3".to_string(),
            "r4".to_string(),
        ], // Eliminates t2 & t5
        profiles: Some(vec![
            // t0 is included in all profiles
            "p1".to_string(),
            "p2".to_string(),
            "p3".to_string(),
            "p5".to_string(),
        ]), // Eliminates t4
    };
    let mut cfg = LazyCachedGlobalConfig::mock_w_parse_err();

    let actual = filter_tendrils(tendrils.clone(), filter, &mut cfg);

    assert_eq!(actual, vec![tendrils[3].clone()]);
}
