use crate::{RawTendril, Tendril, TendrilMode, UniPath};
use rstest::rstest;
use serial_test::serial;
use std::path::{Path, PathBuf};

#[test]
fn invalid_tendril_returns_invalid_tendril() {
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given = RawTendril::new("");
    given.remote = "SomeParentPath1".to_string();

    let actual = given.resolve(&td_repo);

    assert!(actual.is_err());
}

#[rstest]
#[case("<mut-testing>1", "value1")]
#[case("~\\<mut-testing>2", "MyHome\\value2")]
#[case("~/<mut-testing>3", "MyHome/value3")]
#[serial(SERIAL_MUT_ENV_VARS)]
fn vars_and_leading_tilde_in_remote_path_are_resolved(
    #[case] remote: String,
    #[case] expected_remote: PathBuf,
) {
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given = RawTendril::new("SomeLocal");
    given.remote = remote;
    std::env::set_var("mut-testing", "value");
    std::env::set_var("HOME", "MyHome");

    let expected = Tendril::new_expose(
        &td_repo,
        "SomeLocal".into(),
        expected_remote.into(),
        TendrilMode::Overwrite,
    )
    .unwrap();

    let actual = given.resolve(&td_repo).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn var_in_remote_path_doesnt_exist_returns_raw_path() {
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given = RawTendril::new("SomeLocal");
    given.remote = "<I_do_not_exist>".to_string();

    let expected = Tendril::new_expose(
        &td_repo,
        "SomeLocal".into(),
        PathBuf::from("<I_do_not_exist>").into(),
        TendrilMode::Overwrite,
    )
    .unwrap();

    let actual = given.resolve(&td_repo).unwrap();

    assert_eq!(actual, expected);
}

#[test]
#[serial(SERIAL_MUT_ENV_VARS)]
fn leading_tilde_in_remote_path_tilde_value_doesnt_exist_returns_raw_path() {
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given = RawTendril::new("SomeLocal");
    given.remote = "~/SomeRemotePath".to_string();
    std::env::remove_var("HOME");
    std::env::remove_var("HOMEDRIVE");
    std::env::remove_var("HOMEPATH");

    let expected = Tendril::new_expose(
        &td_repo,
        "SomeLocal".into(),
        PathBuf::from("~/SomeRemotePath").into(),
        TendrilMode::Overwrite,
    )
    .unwrap();

    let actual = given.resolve(&td_repo).unwrap();

    assert_eq!(actual, expected);
}

#[rstest]
#[case(TendrilMode::Merge)]
#[case(TendrilMode::Overwrite)]
#[case(TendrilMode::Link)]
fn resolves_tendril_mode_properly(
    #[case] expected_mode: TendrilMode,
) {
    let td_repo = UniPath::from(Path::new("/Repo"));
    let mut given = RawTendril::new("SomeLocal");
    given.remote = "SomeRemotePath".to_string();
    given.mode = expected_mode.clone();

    let expected = Tendril::new_expose(
        &td_repo,
        "SomeLocal".into(),
        PathBuf::from("SomeRemotePath").into(),
        expected_mode,
    )
    .unwrap();

    let actual = given.resolve(&td_repo).unwrap();

    assert_eq!(actual, expected);
}
