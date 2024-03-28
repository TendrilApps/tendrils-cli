use crate::{ActionMode, TendrilBundle};
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
fn string_filter_empty_tests(#[case] value: &[String]) {}

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
    #[case] value: &[String],
    #[case] exp_matches: &[&str],
) {}

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
fn string_filter_non_match_tests(#[case] value: &[String]) {}

#[template]
#[rstest]
#[case("")]
#[case(" ")]
#[case("\n")]
#[case("\t")]
#[case("\r")]
fn supported_weird_values(#[case] value: &str) {}

#[template]
#[rstest]
#[case("*", "\\*")]
#[case("**", "\\*\\*")]
fn supported_asterisk_literals(#[case] value: &str, #[case] pattern: &str) {}

fn samples() -> Vec<TendrilBundle> {
    let mut t0 = TendrilBundle::new("g0", vec!["n0"]);
    t0.link = false;
    t0.profiles = vec![];
    let mut t1 = TendrilBundle::new("g1", vec!["n1"]);
    t1.link = false;
    t1.profiles = vec!["p1".to_string()];
    let mut t2 = TendrilBundle::new("g2", vec!["n2"]);
    t2.link = true;
    t2.profiles = vec!["p2".to_string()];
    let mut t3 = TendrilBundle::new("g3", vec!["n3"]);
    t3.link = false;
    t3.profiles = vec!["p3".to_string()];
    let mut t4 = TendrilBundle::new("g4", vec!["n4"]);
    t4.link = false;
    t4.profiles = vec!["p4".to_string()];

    vec![t0, t1, t2, t3, t4]
}

#[test]
fn empty_tendril_list_returns_empty() {
    let tendrils = vec![];
    let filter = FilterSpec {
        mode: None,
        names: &[],
        profiles: &[],
    };

    let actual = filter_tendrils(tendrils, filter);

    assert!(actual.is_empty())
}

#[test]
fn mode_filter_is_none_does_not_filter_by_mode() {
    let tendrils = samples();
    let filter = FilterSpec {
        mode: None,
        names: &["n0".to_string(), "n1".to_string(), "n2".to_string()],
        profiles: &["p1".to_string(), "p2".to_string()],
    };

    let actual = filter_tendrils(tendrils.clone(), filter);

    assert_eq!(actual, vec![
        tendrils[0].clone(), // Push/pull mode
        tendrils[1].clone(), // Push/pull mode
        tendrils[2].clone(), // Link mode
    ]);
}

#[test]
fn profile_filter_is_empty_does_not_filter_by_profile() {
    let tendrils = samples();
    let filter = FilterSpec {
        mode: Some(ActionMode::Pull),
        names: &["n0".to_string(), "n1".to_string(), "n3".to_string()],
        profiles: &[],
    };

    let actual = filter_tendrils(tendrils.clone(), filter);

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
        names: &[
            "n1".to_string(),
            "n2".to_string(),
            "n3".to_string(),
            "n4".to_string(),
        ], // Eliminates t0
        profiles: &[
            // t0 is included in all profiles
            "p1".to_string(),
            "p2".to_string(),
            "p3".to_string()
        ], // Eliminates t4
    };

    let actual = filter_tendrils(tendrils.clone(), filter);

    assert_eq!(actual, vec![tendrils[1].clone(), tendrils[3].clone()]);
}
