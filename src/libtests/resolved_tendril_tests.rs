use crate::{ResolvedTendril, TendrilMode};
use crate::enums::InvalidTendrilError;
use rstest::rstest;
use std::path::PathBuf;

#[rstest]
#[case("")]
#[case("some/path")]
#[case("some\\path")]
#[case("/somePath")]
#[case("\\somePath")]
#[case("somePath/")]
#[case("somePath\\")]
#[case(".git")]
#[case(".Git")]
#[case(".GIT")]
fn group_is_invalid_returns_invalid_group_error(#[case] group: String) {
    let actual = ResolvedTendril::new(
        group,
        "misc.txt".to_string(),
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
    ).unwrap_err();

    assert!(matches!(actual, InvalidTendrilError::InvalidGroup));
}

#[rstest]
#[case("")]
#[case("some/path")]
#[case("some\\path")]
#[case("/somePath")]
#[case("\\somePath")]
#[case("somePath/")]
#[case("somePath\\")]
fn name_is_invalid_returns_invalid_name_error(#[case] name: String) {
    let actual = ResolvedTendril::new(
        "SomeApp".to_string(),
        name,
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
    ).unwrap_err();

    assert!(matches!(actual, InvalidTendrilError::InvalidName));
}

#[rstest]
#[case("NoDot")]
#[case("single.dot")]
#[case("multi.sandwiched.dots")]
#[case(".LeadingDot")]
#[case("TrailingDot.")]
fn group_is_valid_returns_ok(#[case] group: String) {
    ResolvedTendril::new(
        group,
        "misc.txt".to_string(),
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
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
        TendrilMode::DirOverwrite,
    ).unwrap();
}

#[rstest]
#[case("misc.txt", "ParentPath/", "ParentPath/misc.txt")]
#[case("MiscDir", "ParentPath/", "ParentPath/MiscDir")]
#[case(
    "misc.txt",
    "Crazy`~!@#$%^&*()-_=+|\\[]{}'\";:/?.,<>Symbols/",
    "Crazy`~!@#$%^&*()-_=+|\\[]{}'\";:/?.,<>Symbols/misc.txt"
)]
fn full_path_appends_name_to_parent(
    #[case] name: String,
    #[case] parent: PathBuf,
    #[case] expected: &str,
) {
    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        name,
        parent,
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = tendril.full_path();

    assert_eq!(expected, actual.to_str().unwrap());
}

#[rstest]
#[case("WindowsStyle\\")]
#[case("UnixStyle/")]
#[case("\\Windows\\Style\\")]
#[case("/Unix/Style/")]
#[case("\\Mixed/Style\\")]
#[case("/Mixed\\Style/")]
fn full_path_w_trailing_sep_in_parent_keeps_all_given_seps_regardless_of_curr_platform(
    #[case] parent: &str,
) {
    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        PathBuf::from(parent),
        TendrilMode::DirOverwrite,
    ).unwrap();
    let expected = parent.to_owned() + "misc.txt";

    let actual = tendril.full_path();

    assert_eq!(expected, actual.to_str().unwrap());
}

#[rstest]
#[case("Plain")]
#[case("\\Windows\\Style")]
#[case("/Unix/Style")]
#[case("\\Mixed/Style")]
#[case("/Mixed\\Style")]
fn full_path_wo_trailing_sep_in_parent_uses_curr_platform_sep_only_for_join(
    #[case] parent: &str,
) {
    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        PathBuf::from(parent),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let expected_join_sep = match std::env::consts::OS {
        "windows" => "\\",
        _ => "/",
    };
    let expected = parent.to_owned() + expected_join_sep + "misc.txt";

    let actual = tendril.full_path();

    assert_eq!(expected, actual.to_str().unwrap());
}

#[rstest]
#[case("misc.txt")]
#[case("MiscDir")]
fn full_path_empty_parent_does_not_prepend_dir_sep_to_name(#[case] name: String) {
    let actual = ResolvedTendril::new(
        "SomeApp".to_string(),
        name.clone(),
        PathBuf::from(""),
        TendrilMode::DirOverwrite,
    ).unwrap();

    assert_eq!(actual.full_path(), PathBuf::from(name))
}
