use crate::{Tendril, TendrilMode};
use crate::enums::InvalidTendrilError;
use rstest::rstest;
use rstest_reuse::{self, apply, template};
use std::path::{MAIN_SEPARATOR, PathBuf};

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
#[case("tendrils.json")]
#[case("Tendrils.json")]
#[case("TENDRILS.JSON")]
fn forbidden_groups(#[case] value: &str) {}

#[apply(invalid_groups_and_names)]
fn group_is_invalid_returns_invalid_group_error(#[case] group: &str) {
    let actual = Tendril::new(
        group,
        "misc.txt",
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
    ).unwrap_err();

    assert_eq!(actual, InvalidTendrilError::InvalidGroup);
}

#[apply(forbidden_groups)]
fn group_is_forbidden_returns_invalid_group_error(#[case] group: &str) {
    group_is_invalid_returns_invalid_group_error(group);
}

#[apply(invalid_groups_and_names)]
fn name_is_invalid_returns_invalid_name_error(#[case] name: &str) {
    let actual = Tendril::new(
        "SomeApp",
        name,
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
    ).unwrap_err();

    assert_eq!(actual, InvalidTendrilError::InvalidName);
}

#[rstest]
#[case("")]
#[case("New\nLine")]
#[case("Carriage\rReturn")]
fn parent_is_invalid_returns_invalid_parent_error(#[case] parent: &str) {
    let actual = Tendril::new(
        "SomeApp",
        "misc.txt",
        PathBuf::from(parent),
        TendrilMode::DirOverwrite,
    ).unwrap_err();

    assert_eq!(actual, InvalidTendrilError::InvalidParent);
}

#[apply(valid_groups_and_names)]
fn group_is_valid_returns_ok(#[case] group: &str) {
    Tendril::new(
        group,
        "misc.txt",
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
    ).unwrap();
}

#[apply(valid_groups_and_names)]
fn name_is_valid_returns_ok(#[case] name: &str) {
    Tendril::new(
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
#[case("somePath")]
#[case("/some/path/")]
#[case(" / some / path / ")]
#[case("\\some\\path\\")]
#[case(" \\ some \\ path \\ ")]
fn parent_is_valid_returns_ok(#[case] parent: &str) {
    Tendril::new(
        "SomeApp",
        "misc.txt",
        PathBuf::from(parent),
        TendrilMode::DirOverwrite,
    ).unwrap();
}

#[rstest]
#[case("Plain", &format!("Plain{MAIN_SEPARATOR}misc.txt"))]
#[case("TrailingSep/",  &format!("TrailingSep{MAIN_SEPARATOR}misc.txt"))]
#[case("TrailingSep\\", &format!("TrailingSep{MAIN_SEPARATOR}misc.txt"))]
#[case("/LeadingSep",  &format!("{MAIN_SEPARATOR}LeadingSep{MAIN_SEPARATOR}misc.txt"))]
#[case("\\LeadingSep", &format!("{MAIN_SEPARATOR}LeadingSep{MAIN_SEPARATOR}misc.txt"))]
#[case("/Both/",   &format!("{MAIN_SEPARATOR}Both{MAIN_SEPARATOR}misc.txt"))]
#[case("\\Both\\", &format!("{MAIN_SEPARATOR}Both{MAIN_SEPARATOR}misc.txt"))]
#[case(
    "\\Mixed/Style",
    &format!("{MAIN_SEPARATOR}Mixed{MAIN_SEPARATOR}Style{MAIN_SEPARATOR}misc.txt")
)]
#[case(
    "/Mixed\\Style",
    &format!("{MAIN_SEPARATOR}Mixed{MAIN_SEPARATOR}Style{MAIN_SEPARATOR}misc.txt")
)]
#[case(
    "Crazy`~!@#$%^&*()-_=+|\\[]{}'\";:/?.,<>Symbols/", 
    &format!("Crazy`~!@#$%^&*()-_=+|{MAIN_SEPARATOR}[]{{}}'\";:{MAIN_SEPARATOR}?.,<>Symbols{MAIN_SEPARATOR}misc.txt")
)]
fn full_path_appends_name_to_parent_using_platform_dir_sep_for_all_slashes(
    #[case] parent: PathBuf,
    #[case] expected: &str,
) {
    let tendril = Tendril::new(
        "SomeApp",
        "misc.txt",
        parent,
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = tendril.full_path();

    assert_eq!(expected, actual.to_str().unwrap());
}
