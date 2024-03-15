use crate::{ResolvedTendril, TendrilMode};
use crate::enums::InvalidTendrilError;
use rstest::rstest;
use rstest_reuse::{self, apply, template};
use std::path::PathBuf;

#[template]
#[rstest]
#[case("NoDot")]
#[case("single.dot")]
#[case("multi.sandwiched.dots")]
#[case(".LeadingDot")]
#[case("TrailingDot.")] // Trailing dots are dropped on Windows filesystems,
                        // but a path with a trailing dot will still point to
                        // its equivalent without the dot
fn valid_groups_and_names(#[case] value: &str) {}

#[template]
#[rstest]
#[case("")]
#[case("New\nLine")]
#[case("Carriage\rReturn")]
#[case("some/path")]
#[case("some\\path")]
#[case("/somePath")]
#[case("\\somePath")]
#[case("somePath/")]
#[case("somePath\\")]
fn invalid_groups_and_names(#[case] value: &str) {}

#[template]
#[rstest]
#[case(".git")]
#[case(".Git")]
#[case(".GIT")]
fn forbidden_groups(#[case] value: &str) {}

#[apply(invalid_groups_and_names)]
fn group_is_invalid_returns_invalid_group_error(#[case] group: &str) {
    let actual = ResolvedTendril::new(
        group,
        "misc.txt",
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
    ).unwrap_err();

    assert!(matches!(actual, InvalidTendrilError::InvalidGroup));
}

#[apply(forbidden_groups)]
fn group_is_forbidden_returns_invalid_group_error(#[case] group: &str) {
    group_is_invalid_returns_invalid_group_error(group);
}

#[apply(invalid_groups_and_names)]
fn name_is_invalid_returns_invalid_name_error(#[case] name: &str) {
    let actual = ResolvedTendril::new(
        "SomeApp",
        name,
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
    ).unwrap_err();

    assert!(matches!(actual, InvalidTendrilError::InvalidName));
}

#[rstest]
#[case("New\nLine")]
#[case("Carriage\rReturn")]
fn parent_is_invalid_returns_invalid_parent_error(#[case] parent: &str) {
    let actual = ResolvedTendril::new(
        "SomeApp",
        "misc.txt",
        PathBuf::from(parent),
        TendrilMode::DirOverwrite,
    ).unwrap_err();

    assert!(matches!(actual, InvalidTendrilError::InvalidParent));
}

#[apply(valid_groups_and_names)]
fn group_is_valid_returns_ok(#[case] group: &str) {
    ResolvedTendril::new(
        group,
        "misc.txt",
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
    ).unwrap();
}

#[apply(valid_groups_and_names)]
fn name_is_valid_returns_ok(#[case] name: &str) {
    ResolvedTendril::new(
        "SomeApp",
        name,
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
    ).unwrap();
}

#[apply(forbidden_groups)]
fn name_is_forbidden_group_returns_ok(#[case] name: &str) {
    name_is_valid_returns_ok(name);
}

#[apply(valid_groups_and_names)]
#[case("")]
#[case("somePath")]
#[case("/some/path/")]
#[case(" / some / path / ")]
#[case("\\some\\path\\")]
#[case(" \\ some \\ path \\ ")]
fn parent_is_valid_returns_ok(#[case] group: &str) {
    ResolvedTendril::new(
        "SomeApp",
        "misc.txt",
        PathBuf::from(group),
        TendrilMode::DirOverwrite,
    ).unwrap();
}

#[rstest]
#[case("ParentPath/", "misc.txt", "ParentPath/misc.txt")]
#[case("ParentPath/", "MiscDir", "ParentPath/MiscDir")]
#[case(
    "Crazy`~!@#$%^&*()-_=+|\\[]{}'\";:/?.,<>Symbols/",
    "misc.txt",
    "Crazy`~!@#$%^&*()-_=+|\\[]{}'\";:/?.,<>Symbols/misc.txt"
)]
fn full_path_appends_name_to_parent(
    #[case] parent: PathBuf,
    #[case] name: &str,
    #[case] expected: &str,
) {
    let tendril = ResolvedTendril::new(
        "SomeApp",
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
        "SomeApp",
        "misc.txt",
        PathBuf::from(parent),
        TendrilMode::DirOverwrite,
    ).unwrap();
    let expected = parent.to_owned() + "misc.txt";

    let actual = tendril.full_path();

    assert_eq!(expected, actual.to_str().unwrap());
}

#[rstest]
#[case("Windows\\Style", "Windows\\Style\\misc.txt")]
#[case("Unix/Style", "Unix/Style/misc.txt")]
#[case("\\Windows\\Style", "\\Windows\\Style\\misc.txt")]
#[case("/Unix/Style", "/Unix/Style/misc.txt")]
fn full_path_wo_trailing_sep_in_parent_matches_other_seps_in_parent_for_join_regardless_of_curr_platform(
    #[case] parent: &str,
    #[case] expected: &str,
) {
    let tendril = ResolvedTendril::new(
        "SomeApp",
        "misc.txt",
        PathBuf::from(parent),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = tendril.full_path();

    assert_eq!(expected, actual.to_str().unwrap());
}

#[rstest]
#[case("Plain")]
#[case("\\Mixed/Style")]
#[case("/Mixed\\Style")]
fn full_path_wo_trailing_sep_in_parent_or_mixed_seps_uses_curr_platform_sep_for_join(
    #[case] parent: &str,
) {
    let tendril = ResolvedTendril::new(
        "SomeApp",
        "misc.txt",
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
fn full_path_empty_parent_does_not_prepend_dir_sep_to_name(#[case] name: &str) {
    let actual = ResolvedTendril::new(
        "SomeApp",
        name,
        PathBuf::from(""),
        TendrilMode::DirOverwrite,
    ).unwrap();

    assert_eq!(actual.full_path(), PathBuf::from(name))
}
