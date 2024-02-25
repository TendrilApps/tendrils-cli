use crate::{resolve_tendril, ResolvedTendril, Tendril, TendrilMode};
use crate::test_utils::set_parents;
use rstest::rstest;
use serial_test::serial;
use std::path::PathBuf;

#[rstest]
#[case(true)]
#[case(false)]
fn empty_parent_list_returns_empty(#[case] first_only: bool) {
    let mut given = Tendril::new("SomeApp", vec!["misc.txt"]);

    set_parents(&mut given, &[]);

    let actual = resolve_tendril(given, first_only);

    assert_eq!(actual, vec![]);
}

#[rstest]
#[case("", "misc.txt")]
#[case("SomeApp", "")]
fn invalid_tendril_returns_invalid_tendril(
    #[case] group: &str,
    #[case] name: &str,
) {
    let mut given = Tendril::new(group, vec![name]);
    set_parents(
        &mut given,
        &[
            PathBuf::from("SomeParentPath1"),
            PathBuf::from("SomeParentPath2"),
            PathBuf::from("SomeParentPath3"),
        ]
    );

    let actual = resolve_tendril(given, false);

    assert!(actual[0].is_err());
    assert!(actual[1].is_err());
    assert!(actual[2].is_err());
    assert_eq!(actual.len(), 3);
}

#[rstest]
#[case("", "misc.txt")]
#[case("SomeApp", "")]
fn invalid_tendril_and_empty_parent_list_returns_empty(
    #[case] group: &str,
    #[case] name: &str,
) {
    let mut given = Tendril::new(group, vec![name]);
    set_parents(&mut given, &[]);

    let actual = resolve_tendril(given, false);

    assert!(actual.is_empty());
}

#[test]
fn first_only_true_resolves_first_parent_paths_for_all_names() {
    let mut given = Tendril::new("SomeApp", vec!["misc1.txt", "misc2.txt"]);
    given.dir_merge = false;

    set_parents(
        &mut given,
        &[PathBuf::from("FirstParent"),
        PathBuf::from("SecondParent"),
        PathBuf::from("ThirdParent")]);

    let expected = vec![
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc1.txt".to_string(),
            PathBuf::from("FirstParent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc2.txt".to_string(),
            PathBuf::from("FirstParent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, true);

    assert_eq!(actual, expected);
}

#[test]
fn first_only_false_resolves_all_parent_paths_for_all_names() {
    let mut given = Tendril::new("SomeApp", vec!["misc1.txt", "misc2.txt"]);
    given.dir_merge = false;

    set_parents(
        &mut given,
        &[PathBuf::from("FirstParent"),
        PathBuf::from("SecondParent"),
        PathBuf::from("ThirdParent")]);

    let expected = vec![
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc1.txt".to_string(),
            PathBuf::from("FirstParent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc1.txt".to_string(),
            PathBuf::from("SecondParent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc1.txt".to_string(),
            PathBuf::from("ThirdParent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc2.txt".to_string(),
            PathBuf::from("FirstParent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc2.txt".to_string(),
            PathBuf::from("SecondParent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc2.txt".to_string(),
            PathBuf::from("ThirdParent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, expected);
}

#[test]
fn duplicate_names_resolves_all() {
    let mut given = Tendril::new(
        "SomeApp", vec!["misc.txt", "misc.txt", "misc.txt"]
    );
    given.dir_merge = false;
    set_parents(&mut given, &[PathBuf::from("Parent")]);

    let expected = vec![
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("Parent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("Parent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("Parent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, expected);
}

#[test]
fn duplicate_parent_paths_resolves_all() {
    let mut given = Tendril::new("SomeApp", vec!["misc.txt"]);
    given.dir_merge = false;

    set_parents(
        &mut given,
        &[PathBuf::from("Parent"),
        PathBuf::from("Parent"),
        PathBuf::from("Parent")]);
    let expected = vec![
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("Parent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("Parent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("Parent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn vars_and_leading_tilde_in_parent_path_are_resolved_in_all() {
    let mut given = Tendril::new("SomeApp", vec!["misc.txt"]);
    given.dir_merge = false;
    std::env::set_var("mut-testing", "value");
    std::env::set_var("HOME", "MyHome");

    set_parents(
        &mut given,
        &[
            PathBuf::from("<mut-testing>1"),
            PathBuf::from("~<mut-testing>2"),
            PathBuf::from("~/<mut-testing>3")
        ]
    );
    let expected = vec![
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("value1"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("MyHomevalue2"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("MyHome/value3"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, expected);
}

#[test]
fn var_in_parent_path_doesnt_exist_returns_raw_path() {
    let mut given = Tendril::new("SomeApp", vec!["misc.txt"]);
    given.dir_merge = false;
    set_parents(
        &mut given,
        &[PathBuf::from("<I_do_not_exist>".to_string())],
    );
    let expected = vec![
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("<I_do_not_exist>"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, expected);
}

#[rstest]
#[case("<mut-testing>", "misc.txt")]
#[case("SomeApp", "<mut-testing>")]
#[serial("mut-env-var-testing")]
fn var_in_group_or_name_exists_uses_raw_path(
    #[case] group: &str,
    #[case] name: &str,
) {
    let mut given = Tendril::new(group, vec![name]);
    given.dir_merge = false;
    set_parents(&mut given, &[PathBuf::from("SomeParent")]);
    std::env::set_var("mut-testing", "value");

    let expected = vec![
        Ok(ResolvedTendril::new(
            group.to_string(),
            name.to_string(),
            PathBuf::from("SomeParent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn leading_tilde_in_parent_path_tilde_value_doesnt_exist_returns_raw_path() {
    let mut given = Tendril::new("SomeApp", vec!["misc.txt"]);
    given.dir_merge = false;
    set_parents(
        &mut given,
        &[PathBuf::from("~/SomeParentPath".to_string())],
    );
    std::env::remove_var("HOME");

    let expected = vec![
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("~/SomeParentPath"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, expected);
}

#[rstest]
#[case("~SomeApp", "misc.txt")]
#[case("SomeApp", "~misc.txt")]
#[serial("mut-env-var-testing")]
fn leading_tilde_in_group_or_name_and_tilde_value_exists_uses_raw_path(
    #[case] group: &str,
    #[case] name: &str,
) {
    let mut given = Tendril::new(group, vec![name]);
    given.dir_merge = false;
    set_parents(&mut given, &[PathBuf::from("SomeParent")]);
    std::env::set_var("HOME", "MyHome");

    let expected = vec![
        Ok(ResolvedTendril::new(
            group.to_string(),
            name.to_string(),
            PathBuf::from("SomeParent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, expected);
}

#[rstest]
#[case(true, false, TendrilMode::DirMerge)]
#[case(false, false, TendrilMode::DirOverwrite)]
#[case(true, true, TendrilMode::Link)]
#[case(false, true, TendrilMode::Link)]
fn resolves_tendril_mode_properly(
    #[case] dir_merge: bool,
    #[case] link: bool,
    #[case] expected_mode: TendrilMode,
) {
    let mut given = Tendril::new("SomeApp", vec!["misc.txt"]);
    given.dir_merge = dir_merge;
    given.link = link;
    set_parents(&mut given, &[PathBuf::from("SomeParentPath")]);

    let expected = vec![
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("SomeParentPath"),
            expected_mode,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, true);

    assert_eq!(actual, expected);
}
