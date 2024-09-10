use crate::test_utils::set_parents;
use crate::{Tendril, TendrilBundle, TendrilMode};
use rstest::rstest;
use serial_test::serial;
use std::path::PathBuf;

#[rstest]
#[case(true)]
#[case(false)]
fn empty_parent_list_returns_empty(#[case] first_only: bool) {
    let mut given = TendrilBundle::new("SomeApp", vec!["misc.txt"]);

    set_parents(&mut given, &[]);

    let actual = given.resolve_tendrils(first_only);

    assert_eq!(actual, vec![]);
}

#[rstest]
#[case("", "misc.txt")]
#[case("SomeApp", "")]
fn invalid_tendril_returns_invalid_tendril(
    #[case] group: &str,
    #[case] name: &str,
) {
    let mut given = TendrilBundle::new(group, vec![name]);
    set_parents(&mut given, &[
        PathBuf::from("SomeParentPath1"),
        PathBuf::from("SomeParentPath2"),
        PathBuf::from("SomeParentPath3"),
    ]);

    let actual = given.resolve_tendrils(false);

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
    let mut given = TendrilBundle::new(group, vec![name]);
    set_parents(&mut given, &[]);

    let actual = given.resolve_tendrils(false);

    assert!(actual.is_empty());
}

#[test]
fn first_only_true_resolves_first_parent_paths_for_all_names() {
    let mut given =
        TendrilBundle::new("SomeApp", vec!["misc1.txt", "misc2.txt"]);
    given.dir_merge = false;

    set_parents(&mut given, &[
        PathBuf::from("FirstParent"),
        PathBuf::from("SecondParent"),
        PathBuf::from("ThirdParent"),
    ]);

    let expected = vec![
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc1.txt",
            PathBuf::from("FirstParent").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc2.txt",
            PathBuf::from("FirstParent").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
    ];

    let actual = given.resolve_tendrils(true);

    assert_eq!(actual, expected);
}

#[test]
fn first_only_false_resolves_all_parent_paths_for_all_names() {
    let mut given =
        TendrilBundle::new("SomeApp", vec!["misc1.txt", "misc2.txt"]);
    given.dir_merge = false;

    set_parents(&mut given, &[
        PathBuf::from("FirstParent"),
        PathBuf::from("SecondParent"),
        PathBuf::from("ThirdParent"),
    ]);

    let expected = vec![
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc1.txt",
            PathBuf::from("FirstParent").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc1.txt",
            PathBuf::from("SecondParent").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc1.txt",
            PathBuf::from("ThirdParent").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc2.txt",
            PathBuf::from("FirstParent").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc2.txt",
            PathBuf::from("SecondParent").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc2.txt",
            PathBuf::from("ThirdParent").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
    ];

    let actual = given.resolve_tendrils(false);

    assert_eq!(actual, expected);
}

#[test]
fn duplicate_names_resolves_all() {
    let mut given =
        TendrilBundle::new("SomeApp", vec!["misc.txt", "misc.txt", "misc.txt"]);
    given.dir_merge = false;
    set_parents(&mut given, &[PathBuf::from("Parent")]);

    let expected = vec![
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc.txt",
            PathBuf::from("Parent").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc.txt",
            PathBuf::from("Parent").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc.txt",
            PathBuf::from("Parent").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
    ];

    let actual = given.resolve_tendrils(false);

    assert_eq!(actual, expected);
}

#[test]
fn duplicate_parent_paths_resolves_all() {
    let mut given = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    given.dir_merge = false;

    set_parents(&mut given, &[
        PathBuf::from("Parent"),
        PathBuf::from("Parent"),
        PathBuf::from("Parent"),
    ]);
    let expected = vec![
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc.txt",
            PathBuf::from("Parent").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc.txt",
            PathBuf::from("Parent").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc.txt",
            PathBuf::from("Parent").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
    ];

    let actual = given.resolve_tendrils(false);

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn vars_and_leading_tilde_in_parent_path_are_resolved_in_all() {
    let mut given = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    given.dir_merge = false;
    std::env::set_var("mut-testing", "value");
    std::env::set_var("HOME", "MyHome");

    set_parents(&mut given, &[
        PathBuf::from("<mut-testing>1"),
        PathBuf::from("~\\<mut-testing>2"),
        PathBuf::from("~/<mut-testing>3"),
    ]);
    let expected = vec![
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc.txt",
            PathBuf::from("value1").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc.txt",
            PathBuf::from("MyHome\\value2").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            "SomeApp",
            "misc.txt",
            PathBuf::from("MyHome/value3").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
    ];

    let actual = given.resolve_tendrils(false);

    assert_eq!(actual, expected);
}

#[test]
fn var_in_parent_path_doesnt_exist_returns_raw_path() {
    let mut given = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    given.dir_merge = false;
    set_parents(&mut given, &[PathBuf::from("<I_do_not_exist>".to_string())]);
    let expected = vec![Ok(Tendril::new_expose(
        "SomeApp",
        "misc.txt",
        PathBuf::from("<I_do_not_exist>").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap())];

    let actual = given.resolve_tendrils(false);

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn leading_tilde_in_parent_path_tilde_value_doesnt_exist_returns_raw_path() {
    let mut given = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    given.dir_merge = false;
    set_parents(&mut given, &[PathBuf::from("~/SomeParentPath".to_string())]);
    std::env::remove_var("HOME");
    std::env::remove_var("HOMEDRIVE");
    std::env::remove_var("HOMEPATH");

    let expected = vec![Ok(Tendril::new_expose(
        "SomeApp",
        "misc.txt",
        PathBuf::from("~/SomeParentPath").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap())];

    let actual = given.resolve_tendrils(false);

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
    let mut given = TendrilBundle::new("SomeApp", vec!["misc.txt"]);
    given.dir_merge = dir_merge;
    given.link = link;
    set_parents(&mut given, &[PathBuf::from("SomeParentPath")]);

    let expected = vec![Ok(Tendril::new_expose(
        "SomeApp",
        "misc.txt",
        PathBuf::from("SomeParentPath").into(),
        expected_mode,
    )
    .unwrap())];

    let actual = given.resolve_tendrils(true);

    assert_eq!(actual, expected);
}
