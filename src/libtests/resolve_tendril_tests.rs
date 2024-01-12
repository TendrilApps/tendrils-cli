use crate::errors::ResolveTendrilError;
use crate::{resolve_tendril, ResolvedTendril, Tendril, TendrilMode};
use crate::test_utils::{set_all_platform_paths, get_username};
use rstest::rstest;
use serial_test::serial;
use std::path::PathBuf;

#[test]
fn empty_parent_list_returns_empty() {
    let mut given = Tendril::new("SomeApp", "misc.txt");

    set_all_platform_paths(&mut given, &[]);

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, vec![]);
}

#[rstest]
#[case("", "misc.txt")]
#[case("SomeApp", "")]
fn invalid_tendril_returns_invalid_tendril(
    #[case] app: &str,
    #[case] name: &str,
) {
    let mut given = Tendril::new(app, name);
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
    #[case] app: &str,
    #[case] name: &str,
) {
    let mut given = Tendril::new(app, name);
    set_all_platform_paths(&mut given, &[]);

    let actual = resolve_tendril(given, false);

    assert!(actual.is_empty());
}

#[test]
fn first_only_true_resolves_first_of_multiple_parent_paths() {
    let mut given = Tendril::new("SomeApp", "misc.txt");
    given.folder_merge = false;
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
            TendrilMode::FolderOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, true);

    assert_eq!(actual, expected);
}

#[test]
fn resolves_all_of_multiple_parent_paths() {
    let mut given = Tendril::new("SomeApp", "misc.txt");
    given.folder_merge = false;

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
            TendrilMode::FolderOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("SecondParent"),
            TendrilMode::FolderOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("ThirdParent"),
            TendrilMode::FolderOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, expected);
}

#[test]
fn duplicate_parent_paths_resolves_all() {
    let mut given = Tendril::new("SomeApp", "misc.txt");
    given.folder_merge = false;

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
            TendrilMode::FolderOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("Parent"),
            TendrilMode::FolderOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from("Parent"),
            TendrilMode::FolderOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, expected);
}

#[test]
#[serial]
fn supported_variables_are_resolved_in_all() {
    let mut given = Tendril::new("SomeApp", "misc.txt");
    given.folder_merge = false;
    let mut expected_parent1 = get_username();
    expected_parent1.push('1');
    let mut expected_parent2 = get_username();
    expected_parent2.push('2');
    let mut expected_parent3 = get_username();
    expected_parent3.push('3');

    set_all_platform_paths(
        &mut given,
        &[
            PathBuf::from("<user>1"),
            PathBuf::from("<user>2"),
            PathBuf::from("<user>3")
        ]
    );
    let expected = vec![
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from(expected_parent1),
            TendrilMode::FolderOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from(expected_parent2),
            TendrilMode::FolderOverwrite,
        ).unwrap()),
        Ok(ResolvedTendril::new(
            "SomeApp".to_string(),
            "misc.txt".to_string(),
            PathBuf::from(expected_parent3),
            TendrilMode::FolderOverwrite,
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, false);

    assert_eq!(actual, expected);
}

#[test]
fn resolves_paths_for_current_platform() {
    let given = Tendril {
        app: "SomeApp".to_string(),
        name: "misc.txt".to_string(),
        parent_dirs_mac: ["MacParent".to_string()].to_vec(),
        parent_dirs_windows: ["WinParent".to_string()].to_vec(),
        folder_merge: false,
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
            TendrilMode::FolderOverwrite
        ).unwrap()),
    ];

    let actual = resolve_tendril(given, true);

    assert_eq!(actual, expected);
}
