use crate::enums::ResolveTendrilError;
use crate::{resolve_tendril, ResolvedTendril, Tendril, TendrilMode};
use crate::test_utils::set_all_platform_paths;
use rstest::rstest;
use serial_test::serial;
use std::path::PathBuf;

#[rstest]
#[case(true)]
#[case(false)]
fn empty_parent_list_returns_empty(#[case] first_only: bool) {
    let mut given = Tendril::new("SomeApp", "misc.txt");

    set_all_platform_paths(&mut given, &[]);

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
    let mut given = Tendril::new(group, name);
    set_all_platform_paths(
        &mut given,
        &[
            PathBuf::from("SomeParentPath1"),
            PathBuf::from("SomeParentPath2"),
            PathBuf::from("SomeParentPath3"),
        ]
    );

    let actual = resolve_tendril(given, false);

    assert!(matches!(actual[0], Err(ResolveTendrilError::InvalidTendril(_))));
    assert!(matches!(actual[1], Err(ResolveTendrilError::InvalidTendril(_))));
    assert!(matches!(actual[2], Err(ResolveTendrilError::InvalidTendril(_))));
    assert_eq!(actual.len(), 3);
}

#[rstest]
#[case("", "misc.txt")]
#[case("SomeApp", "")]
fn invalid_tendril_and_empty_parent_list_returns_empty(
    #[case] group: &str,
    #[case] name: &str,
) {
    let mut given = Tendril::new(group, name);
    set_all_platform_paths(&mut given, &[]);

    let actual = resolve_tendril(given, false);

    assert!(actual.is_empty());
}

#[test]
fn first_only_true_resolves_first_of_multiple_parent_paths() {
    let mut given = Tendril::new("SomeApp", "misc.txt");
    given.dir_merge = false;
    set_all_platform_paths(
        &mut given,
        &[PathBuf::from("FirstParent"),
        PathBuf::from("SecondParent"),
        PathBuf::from("ThirdParent")]);
    let expected = vec![
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("FirstParent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, true);

    assert_eq!(actual, expected);
}

#[test]
fn resolves_all_of_multiple_parent_paths() {
    let mut given = Tendril::new("SomeApp", "misc.txt");
    given.dir_merge = false;

    set_all_platform_paths(
        &mut given,
        &[PathBuf::from("FirstParent"),
        PathBuf::from("SecondParent"),
        PathBuf::from("ThirdParent")]);
    let expected = vec![
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("FirstParent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("SecondParent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("ThirdParent"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, expected);
}

#[test]
fn duplicate_parent_paths_resolves_all() {
    let mut given = Tendril::new("SomeApp", "misc.txt");
    given.dir_merge = false;

    set_all_platform_paths(
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
fn vars_in_parent_path_are_resolved_in_all() {
    let mut given = Tendril::new("SomeApp", "misc.txt");
    given.dir_merge = false;
    std::env::set_var("mut-testing", "value");

    set_all_platform_paths(
        &mut given,
        &[
            PathBuf::from("<mut-testing>1"),
            PathBuf::from("<mut-testing>2"),
            PathBuf::from("<mut-testing>3")
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
            PathBuf::from("value2"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("value3"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, expected);
}

#[test]
fn var_in_parent_path_doesnt_exist_returns_raw_path() {
    let mut given = Tendril::new("SomeApp", "misc.txt");
    given.dir_merge = false;
    set_all_platform_paths(
        &mut given,
        &[
            PathBuf::from("SomeParentPath1".to_string()),
            PathBuf::from("<I_do_not_exist>".to_string()),
            PathBuf::from("SomeParentPath3".to_string()),
        ],
    );
    let expected = vec![
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("SomeParentPath1"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("<I_do_not_exist>"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("SomeParentPath3"),
            TendrilMode::DirOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, expected);
}

#[test]
fn resolves_paths_for_current_platform() {
    let given = Tendril {
        group: "SomeApp".to_string(),
        name: "misc.txt".to_string(),
        parent_dirs_mac: ["MacParent".to_string()].to_vec(),
        parent_dirs_windows: ["WinParent".to_string()].to_vec(),
        dir_merge: false,
        link: false,
    };

    let expected_parent = match std::env::consts::OS {
        "macos" => PathBuf::from(&given.parent_dirs_mac[0]),
        "windows" => PathBuf::from(&given.parent_dirs_windows[0]),
        _ => unimplemented!()
    };
    let expected = vec![
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            expected_parent,
            TendrilMode::DirOverwrite
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, true);

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
    let mut given = Tendril::new("SomeApp", "misc.txt");
    given.dir_merge = dir_merge;
    given.link = link;
    set_all_platform_paths(&mut given, &[PathBuf::from("SomeParentPath")]);

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
