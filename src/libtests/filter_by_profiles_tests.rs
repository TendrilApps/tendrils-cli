use crate::{filter_by_profiles, Tendril};
use rstest::rstest;

#[rstest]
#[case(vec![])]
#[case(vec!["".to_string()])]
#[case(vec![" ".to_string()])]
#[case(vec!["*".to_string()])]
#[case(vec!["**".to_string()])]
#[case(vec!["p1".to_string()])]
#[case(vec!["p2".to_string()])]
#[case(vec!["**p1**".to_string()])]
#[case(vec!["p1".to_string(), "p2".to_string()])]
fn tendril_with_empty_profiles_included_in_all(
    #[case] given_filters: Vec<String>
) {
    let t1 = Tendril::new("SomeApp", "misc.txt");
    assert!(t1.profiles.is_empty());
    let t2 = Tendril::new("SomeApp", "misc2.txt");
    assert!(t2.profiles.is_empty());
    let tendrils = [t1.clone(), t2.clone()];

    let actual = filter_by_profiles(&tendrils, &given_filters);

    assert_eq!(actual, vec![t1, t2]);
}

#[rstest]
#[case(vec!["p1".to_string()])]
#[case(vec!["p2".to_string()])]
#[case(vec!["".to_string(), "p1".to_string()])]
#[case(vec!["p1".to_string(), "p2".to_string()])]
#[case(vec!["p1".to_string(), "p3".to_string()])]
#[case(vec!["p2".to_string(), "p3".to_string()])]
#[case(vec!["p1".to_string(), "p2".to_string(), "p3".to_string()])]
fn tendril_only_included_if_any_profile_matches_exactly(
    #[case] given_filters: Vec<String>
) {
    let mut t1 = Tendril::new("SomeApp", "misc.txt");
    t1.profiles = vec!["p1".to_string(), "p2".to_string()];
    let mut t2 = Tendril::new("SomeApp", "misc2.txt");
    t2.profiles = vec!["p4".to_string()];
    let tendrils = [t1.clone(), t2.clone()];

    let actual = filter_by_profiles(&tendrils, &given_filters);

    assert_eq!(actual, vec![t1]);
}

#[rstest]
#[case(vec!["".to_string()])]
#[case(vec!["*".to_string()])]
#[case(vec!["**".to_string()])]
#[case(vec!["P1".to_string()])]
#[case(vec!["P2".to_string()])]
#[case(vec!["p3".to_string()])]
#[case(vec!["p1 ".to_string()])]
#[case(vec![" p1".to_string()])]
#[case(vec![" p1".to_string()])]
#[case(vec!["*p1*".to_string()])]
#[case(vec!["**p1**".to_string()])]
#[case(vec!["p1Leading".to_string()])]
#[case(vec!["Trailingp1".to_string()])]
#[case(vec!["P1".to_string(), "P2".to_string(), "p3".to_string()])]
fn tendril_not_included_if_not_empty_and_no_profile_matches_exactly(
    #[case] given_filters: Vec<String>
) {
    let mut t1 = Tendril::new("SomeApp", "misc.txt");
    t1.profiles = vec!["p1".to_string(), "p2".to_string()];
    let t2 = Tendril::new("SomeApp", "misc2.txt");
    assert!(t2.profiles.is_empty());
    let tendrils = [t1.clone(), t2.clone()];

    let actual = filter_by_profiles(&tendrils, &given_filters);

    assert_eq!(actual, vec![t2]);
}

#[test]
fn duplicate_filter_profiles_only_returns_tendril_once() {
    let mut t1 = Tendril::new("SomeApp", "misc.txt");
    t1.profiles = vec!["p1".to_string()];
    let mut t2 = Tendril::new("SomeApp", "misc2.txt");
    t2.profiles = vec!["p2".to_string()];
    let tendrils = [t1.clone(), t2.clone()];
    let given_filters = [
        "p1".to_string(),
        "p1".to_string(),
        "p1".to_string(),
    ];

    let actual = filter_by_profiles(&tendrils, &given_filters);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn duplicate_tendril_profiles_only_returns_tendril_once() {
    let mut t1 = Tendril::new("SomeApp", "misc.txt");
    t1.profiles = vec![
        "p1".to_string(),
        "p1".to_string(),
        "p1".to_string(),
    ];
    let mut t2 = Tendril::new("SomeApp", "misc2.txt");
    t2.profiles = vec!["p2".to_string()];
    let tendrils = [t1.clone(), t2.clone()];
    let given_filters = ["p1".to_string()];

    let actual = filter_by_profiles(&tendrils, &given_filters);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn duplicate_tendrils_returns_all_instances() {
    let mut t1 = Tendril::new("SomeApp", "misc.txt");
    t1.profiles = vec!["p1".to_string()];
    let mut t2 = Tendril::new("SomeApp", "misc2.txt");
    t2.profiles = vec!["p2".to_string()];
    let tendrils = [t1.clone(), t1.clone(), t1.clone(), t2.clone()];
    let given_filters = ["p1".to_string()];

    let actual = filter_by_profiles(&tendrils, &given_filters);

    assert_eq!(actual, vec![t1.clone(), t1.clone(), t1]);
}

#[rstest]
#[case("")]
#[case(" ")]
#[case("*")]
#[case("**")]
#[case("\n")]
#[case("\t")]
#[case("\r")]
fn weird_profile_names_are_supported(
    #[case] profile: String
) {
    let mut t1 = Tendril::new("SomeApp", "misc.txt");
    t1.profiles = vec![profile.clone()];
    let mut t2 = Tendril::new("SomeApp", "misc2.txt");
    t2.profiles = vec!["p1".to_string()];
    let tendrils = [t1.clone(), t2.clone()];

    let actual = filter_by_profiles(&tendrils, &[profile]);

    assert_eq!(actual, vec![t1]);
}

#[test]
fn empty_filters_list_returns_all_tendrils() {
    let mut t1 = Tendril::new("SomeApp", "misc.txt");
    t1.profiles = vec!["p1".to_string()];
    let mut t2 = Tendril::new("SomeApp", "misc2.txt");
    t2.profiles = vec![];
    let tendrils = [t1.clone(), t2.clone()];

    let actual = filter_by_profiles(&tendrils, &[]);

    assert_eq!(actual, vec![t1, t2]);
}
