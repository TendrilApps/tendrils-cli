use crate::test_utils::set_remotes;
use crate::{Tendril, TendrilBundle, TendrilMode, UniPath};
use rstest::rstest;
use serial_test::serial;
use std::path::{Path, PathBuf};

#[rstest]
#[case(true)]
#[case(false)]
fn empty_parent_list_returns_empty(#[case] first_only: bool) {
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given = TendrilBundle::new("SomeLocal");

    set_remotes(&mut given, &[]);

    let actual = given.resolve_tendrils(&td_repo, first_only);

    assert_eq!(actual, vec![]);
}

#[test]
fn invalid_tendril_returns_invalid_tendril() {
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given = TendrilBundle::new("");
    set_remotes(&mut given, &[
        PathBuf::from("SomeParentPath1"),
        PathBuf::from("SomeParentPath2"),
        PathBuf::from("SomeParentPath3"),
    ]);

    let actual = given.resolve_tendrils(&td_repo, false);

    assert!(actual[0].is_err());
    assert!(actual[1].is_err());
    assert!(actual[2].is_err());
    assert_eq!(actual.len(), 3);
}

#[test]
fn invalid_tendril_and_empty_parent_list_returns_empty() {
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given = TendrilBundle::new("");
    set_remotes(&mut given, &[]);

    let actual = given.resolve_tendrils(&td_repo, false);

    assert!(actual.is_empty());
}

#[test]
fn first_only_true_resolves_first_remote_path_only() {
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given =
        TendrilBundle::new("SomeLocal");
    given.dir_merge = false;

    set_remotes(&mut given, &[
        PathBuf::from("FirstRemote"),
        PathBuf::from("SecondRemote"),
        PathBuf::from("ThirdRemote"),
    ]);

    let expected = vec![
        Ok(Tendril::new_expose(
            &td_repo,
            "SomeLocal".into(),
            PathBuf::from("FirstRemote").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
    ];

    let actual = given.resolve_tendrils(&td_repo, true);

    assert_eq!(actual, expected);
}

#[test]
fn first_only_false_resolves_all_remote_paths() {
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given = TendrilBundle::new("SomeLocal");
    given.dir_merge = false;

    set_remotes(&mut given, &[
        PathBuf::from("FirstRemote"),
        PathBuf::from("SecondRemote"),
        PathBuf::from("ThirdRemote"),
    ]);

    let expected = vec![
        Ok(Tendril::new_expose(
            &td_repo,
            "SomeLocal".into(),
            PathBuf::from("FirstRemote").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            &td_repo,
            "SomeLocal".into(),
            PathBuf::from("SecondRemote").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            &td_repo,
            "SomeLocal".into(),
            PathBuf::from("ThirdRemote").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
    ];

    let actual = given.resolve_tendrils(&td_repo, false);

    assert_eq!(actual, expected);
}

#[test]
fn duplicate_remotes_resolves_all() {
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given = TendrilBundle::new("SomeLocal");
    given.dir_merge = false;

    set_remotes(&mut given, &[
        PathBuf::from("Remote"),
        PathBuf::from("Remote"),
        PathBuf::from("Remote"),
    ]);
    let expected = vec![
        Ok(Tendril::new_expose(
            &td_repo,
            "SomeLocal".into(),
            PathBuf::from("Remote").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            &td_repo,
            "SomeLocal".into(),
            PathBuf::from("Remote").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            &td_repo,
            "SomeLocal".into(),
            PathBuf::from("Remote").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
    ];

    let actual = given.resolve_tendrils(&td_repo, false);

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn vars_and_leading_tilde_in_remote_path_are_resolved_in_all() {
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given = TendrilBundle::new("SomeLocal");
    given.dir_merge = false;
    std::env::set_var("mut-testing", "value");
    std::env::set_var("HOME", "MyHome");

    set_remotes(&mut given, &[
        PathBuf::from("<mut-testing>1"),
        PathBuf::from("~\\<mut-testing>2"),
        PathBuf::from("~/<mut-testing>3"),
    ]);
    let expected = vec![
        Ok(Tendril::new_expose(
            &td_repo,
            "SomeLocal".into(),
            PathBuf::from("value1").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            &td_repo,
            "SomeLocal".into(),
            PathBuf::from("MyHome\\value2").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
        Ok(Tendril::new_expose(
            &td_repo,
            "SomeLocal".into(),
            PathBuf::from("MyHome/value3").into(),
            TendrilMode::DirOverwrite,
        )
        .unwrap()),
    ];

    let actual = given.resolve_tendrils(&td_repo, false);

    assert_eq!(actual, expected);
}

#[test]
fn var_in_remote_path_doesnt_exist_returns_raw_path() {
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given = TendrilBundle::new("SomeLocal");
    given.dir_merge = false;
    set_remotes(&mut given, &[PathBuf::from("<I_do_not_exist>".to_string())]);
    let expected = vec![Ok(Tendril::new_expose(
        &td_repo,
        "SomeLocal".into(),
        PathBuf::from("<I_do_not_exist>").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap())];

    let actual = given.resolve_tendrils(&td_repo, false);

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn leading_tilde_in_remote_path_tilde_value_doesnt_exist_returns_raw_path() {
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given = TendrilBundle::new("SomeLocal");
    given.dir_merge = false;
    set_remotes(&mut given, &[PathBuf::from("~/SomeRemotePath".to_string())]);
    std::env::remove_var("HOME");
    std::env::remove_var("HOMEDRIVE");
    std::env::remove_var("HOMEPATH");

    let expected = vec![Ok(Tendril::new_expose(
        &td_repo,
        "SomeLocal".into(),
        PathBuf::from("~/SomeRemotePath").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap())];

    let actual = given.resolve_tendrils(&td_repo, false);

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
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given = TendrilBundle::new("SomeLocal");
    given.dir_merge = dir_merge;
    given.link = link;
    set_remotes(&mut given, &[PathBuf::from("SomeRemotePath")]);

    let expected = vec![Ok(Tendril::new_expose(
        &td_repo,
        "SomeLocal".into(),
        PathBuf::from("SomeRemotePath").into(),
        expected_mode,
    )
    .unwrap())];

    let actual = given.resolve_tendrils(&td_repo, true);

    assert_eq!(actual, expected);
}
