use crate::{ResolvedTendril, TendrilMode};
use crate::errors::InvalidTendrilError;
use rstest::rstest;
use std::path::PathBuf;

#[rstest]
#[case("")]
#[case("some/path")]
#[case("some\\path")]
#[case(".git")]
#[case(".Git")]
#[case(".GIT")]
fn app_is_invalid_returns_invalid_app_error(#[case] app: String) {
    let actual = ResolvedTendril::new(
        app,
        "misc.txt".to_string(),
        PathBuf::from("SomePath"),
        TendrilMode::FolderOverwrite,
    ).unwrap_err();

    assert!(matches!(actual, InvalidTendrilError::InvalidApp));
}

#[rstest]
#[case("")]
#[case("some/path")]
#[case("some\\path")]
fn name_is_invalid_returns_invalid_name_error(#[case] name: String) {
    let actual = ResolvedTendril::new(
        "SomeApp".to_string(),
        name,
        PathBuf::from("SomePath"),
        TendrilMode::FolderOverwrite,
    ).unwrap_err();

    assert!(matches!(actual, InvalidTendrilError::InvalidName));
}

#[rstest]
#[case("NoDot")]
#[case("single.dot")]
#[case("multi.sandwiched.dots")]
#[case(".LeadingDot")]
#[case("TrailingDot.")]
fn app_is_valid_returns_ok(#[case] app: String) {
    ResolvedTendril::new(
        app,
        "misc.txt".to_string(),
        PathBuf::from("SomePath"),
        TendrilMode::FolderOverwrite,
    ).unwrap();
}

#[rstest]
#[case("NoDot")]
#[case("single.dot")]
#[case("multi.sandwiched.dots")]
#[case(".LeadingDot")]
#[case("TrailingDot.")]
#[case(".git")]
#[case(".Git")]
#[case(".GIT")]
fn name_is_valid_returns_ok(#[case] name: String) {
    ResolvedTendril::new(
        "SomeApp".to_string(),
        name,
        PathBuf::from("SomePath"),
        TendrilMode::FolderOverwrite,
    ).unwrap();
}

#[rstest]
#[case("misc.txt", "ParentPath")]
#[case("MiscDir", "ParentPath")]
#[case("misc.txt", "Crazy`~!@#$%^&*()-_=+|\\[]{}'\";:/?.,<>Symbols")]
fn full_path_appends_name_to_parent(#[case] name: String, #[case] parent: PathBuf) {
    let actual = ResolvedTendril::new(
        "SomeApp".to_string(),
        name,
        parent,
        TendrilMode::FolderOverwrite,
    ).unwrap();

    assert_eq!(actual.full_path(), actual.parent.join(actual.name()))
}
