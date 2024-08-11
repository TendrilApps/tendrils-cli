use crate::{InvalidTendrilError, Tendril, TendrilMode};
use rstest::rstest;
use rstest_reuse::{self, apply, template};
use std::path::{PathBuf, MAIN_SEPARATOR as SEP};

#[template]
#[rstest]
#[case("NoDot")]
#[case("single.dot")]
#[case("multi.sandwiched.dots")]
#[case(".LeadingDot")]
#[case("TrailingDot.")] // Trailing dots are dropped on Windows filesystems,
                        // but a path with a trailing dot will still point to
                        // its equivalent without the dot
#[case("tendrils.json")]
#[case("Tendrils.json")]
#[case("TENDRILS.JSON")]
fn valid_groups_and_names(#[case] value: &str) {}

#[template]
#[rstest]
#[case("some/path")]
#[case("some\\path")]
#[case("/somePath")]
#[case("\\somePath")]
#[case("somePath/")]
#[case("somePath\\")]
fn valid_names_but_invalid_groups(#[case] value: &str) {}

#[template]
#[rstest]
#[case("")]
#[case("New\nLine")]
#[case("Carriage\rReturn")]
fn invalid_groups_and_names(#[case] value: &str) {}

#[template]
#[rstest]
#[case(".git")]
#[case(".Git")]
#[case(".GIT")]
#[case(".tendrils")]
#[case(".Tendrils")]
#[case(".TENDRILS")]
fn forbidden_groups(#[case] value: &str) {}

#[apply(invalid_groups_and_names)]
fn group_is_invalid_returns_invalid_group_error(#[case] group: &str) {
    let actual = Tendril::new_expose(
        group,
        "misc.txt",
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
    )
    .unwrap_err();

    assert_eq!(actual, InvalidTendrilError::InvalidGroup);
}

#[apply(valid_names_but_invalid_groups)]
fn group_is_invalid_returns_invalid_group_error_2(#[case] group: &str) {
    group_is_invalid_returns_invalid_group_error(group);
}

#[apply(forbidden_groups)]
fn group_is_forbidden_returns_invalid_group_error(#[case] group: &str) {
    group_is_invalid_returns_invalid_group_error(group);
}

#[apply(invalid_groups_and_names)]
fn name_is_invalid_returns_invalid_name_error(#[case] name: &str) {
    let actual = Tendril::new_expose(
        "SomeApp",
        name,
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
    )
    .unwrap_err();

    assert_eq!(actual, InvalidTendrilError::InvalidName);
}

#[rstest]
#[case("")]
#[case("New\nLine")]
#[case("Carriage\rReturn")]
fn parent_is_invalid_returns_invalid_parent_error(#[case] parent: &str) {
    let actual = Tendril::new_expose(
        "SomeApp",
        "misc.txt",
        PathBuf::from(parent),
        TendrilMode::DirOverwrite,
    )
    .unwrap_err();

    assert_eq!(actual, InvalidTendrilError::InvalidParent);
}

#[apply(valid_groups_and_names)]
fn group_is_valid_returns_ok(#[case] group: &str) {
    Tendril::new_expose(
        group,
        "misc.txt",
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
    )
    .unwrap();
}

#[apply(valid_groups_and_names)]
fn name_is_valid_returns_ok(#[case] name: &str) {
    Tendril::new_expose(
        "SomeApp",
        name,
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
    )
    .unwrap();
}

#[apply(valid_names_but_invalid_groups)]
fn name_is_valid_returns_ok_2(#[case] name: &str) {
    name_is_valid_returns_ok(name);
}

#[apply(forbidden_groups)]
fn name_is_forbidden_group_returns_ok(#[case] name: &str) {
    name_is_valid_returns_ok(name);
}

#[apply(forbidden_groups)]
fn name_subdir_is_forbidden_group_returns_ok(#[case] subdir_name: &str) {
    Tendril::new_expose(
        "SomeApp",
        &format!("{subdir_name}/misc.txt"),
        PathBuf::from("SomePath"),
        TendrilMode::DirOverwrite,
    )
    .unwrap();
}

#[apply(valid_groups_and_names)]
#[case("somePath")]
#[case("/some/path/")]
#[case(" / some / path / ")]
#[case("\\some\\path\\")]
#[case(" \\ some \\ path \\ ")]
fn parent_is_valid_returns_ok(#[case] parent: &str) {
    Tendril::new_expose(
        "SomeApp",
        "misc.txt",
        PathBuf::from(parent),
        TendrilMode::DirOverwrite,
    )
    .unwrap();
}

#[rstest]
#[case("Plain", &format!("Plain{SEP}"))]
#[case("TrailingSep/", &format!("TrailingSep{SEP}"))]
#[case("TrailingSep\\", &format!("TrailingSep{SEP}"))]
#[case("DblTrailingSep//", &format!("DblTrailingSep{SEP}{SEP}"))]
#[case("DblTrailingSep\\\\", &format!("DblTrailingSep{SEP}{SEP}"))]
#[case("/LeadingSep",  &format!("{SEP}LeadingSep{SEP}"))]
#[case("\\LeadingSep", &format!("{SEP}LeadingSep{SEP}"))]
#[case("/Both/",   &format!("{SEP}Both{SEP}"))]
#[case("\\Both\\", &format!("{SEP}Both{SEP}"))]
#[case("\\Mixed/Style", &format!("{SEP}Mixed{SEP}Style{SEP}"))]
#[case("/Mixed\\Style", &format!("{SEP}Mixed{SEP}Style{SEP}"))]
#[case(
    "Crazy`~!@#$%^&*()-_=+|\\[]{}'\";:/?.,<>Symbols/",
    &format!("Crazy`~!@#$%^&*()-_=+|{SEP}[]{{}}'\";:{SEP}?.,<>Symbols{SEP}")
)]
fn full_path_appends_name_to_parent_using_platform_dir_sep_for_all_slashes(
    #[case] parent: PathBuf,

    // (given, expected after appending to parent)
    #[values(
        ("misc.txt", "misc.txt".to_string()),
        ("misc/nested.txt",  format!("misc{SEP}nested.txt")),
        ("misc\\nested.txt", format!("misc{SEP}nested.txt")),
        ("trailingSep/",  format!("trailingSep{SEP}")),
        ("trailingSep\\", format!("trailingSep{SEP}")),
        ("/leadingSep.txt",  "leadingSep.txt".to_string()),
        ("\\leadingSep.txt", "leadingSep.txt".to_string()),
        ("//dblLeadingSep.txt",  format!("{SEP}dblLeadingSep.txt")),
        ("\\\\dblLeadingSep.txt", format!("{SEP}dblLeadingSep.txt")),
        ("/Both/",   format!("Both{SEP}")),
        ("\\Both\\", format!("Both{SEP}")),
        ("/Mixed\\misc.txt", format!("Mixed{SEP}misc.txt")),
        ("\\Mixed/misc.txt", format!("Mixed{SEP}misc.txt")),
        ("C:\\Absolute\\Path", format!("C:{SEP}Absolute{SEP}Path")),
    )]
    name_given_and_exp: (&str, String),

    #[case] expected_prefix: &str,
) {
    let tendril = Tendril::new_expose(
        "SomeApp",
        name_given_and_exp.0,
        parent,
        TendrilMode::DirOverwrite,
    )
    .unwrap();
    let expected = format!("{expected_prefix}{}", name_given_and_exp.1);

    let actual = tendril.full_path();

    assert_eq!(expected, actual.to_str().unwrap());
}
