use crate::{InvalidTendrilError, Tendril, TendrilMode, UniPath};
use crate::test_utils::non_utf_8_text;
use rstest::rstest;
use rstest_reuse::{self, apply, template};
use serial_test::serial;
use std::path::{
    Path,
    PathBuf,
    MAIN_SEPARATOR as SEP,
    MAIN_SEPARATOR_STR as SEP_STR,
};

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
pub fn valid_groups_and_names(#[case] value: &str) {}

#[template]
#[rstest]
#[case("some/path")]
#[case("some\\path")]
#[case("/somePath")]
#[case("\\somePath")]
#[case("somePath/")]
#[case("somePath\\")]
pub fn valid_names_but_invalid_groups(#[case] value: &str) {}

#[template]
#[rstest]
#[case("")]
#[case("New\nLine")]
#[case("Carriage\rReturn")]
pub fn invalid_groups_and_names(#[case] value: &str) {}

#[template]
#[rstest]
#[case(".git")]
#[case(".Git")]
#[case(".GIT")]
#[case(".tendrils")]
#[case(".Tendrils")]
#[case(".TENDRILS")]
pub fn forbidden_groups(#[case] value: &str) {}

#[apply(invalid_groups_and_names)]
fn group_is_invalid_returns_invalid_group_error(#[case] group: &str) {
    let actual = Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        group,
        "misc.txt",
        PathBuf::from("SomePath").into(),
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
        &UniPath::from(Path::new("/Repo")),
        "SomeApp",
        name,
        PathBuf::from("SomePath").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap_err();

    assert_eq!(actual, InvalidTendrilError::InvalidName);
}

#[rstest]
#[case("New\nLine")]
#[case("Carriage\rReturn")]
fn parent_is_invalid_returns_invalid_parent_error(#[case] parent: &str) {
    let actual = Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        "SomeApp",
        "misc.txt",
        PathBuf::from(parent).into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap_err();

    assert_eq!(actual, InvalidTendrilError::InvalidParent);
}

#[apply(valid_groups_and_names)]
fn group_is_valid_returns_ok(#[case] group: &str) {
    Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        group,
        "misc.txt",
        PathBuf::from("SomePath").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();
}

#[apply(valid_groups_and_names)]
fn name_is_valid_returns_ok(#[case] name: &str) {
    Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        "SomeApp",
        name,
        PathBuf::from("SomePath").into(),
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
        &UniPath::from(Path::new("/Repo")),
        "SomeApp",
        &format!("{subdir_name}/misc.txt"),
        PathBuf::from("SomePath").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();
}

#[apply(valid_groups_and_names)]
#[case("/")]
#[case("\\")]
#[case("somePath")]
#[case("/some/path/")]
#[case("\\some\\path\\")]
#[case(" / some / path / ")]
#[case(" \\ some \\ path \\ ")]
fn parent_is_valid_returns_ok(#[case] parent: &str) {
    Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        "SomeApp",
        "misc.txt",
        PathBuf::from(parent).into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();
}

#[rstest]
#[case("", "Plain", &format!("{SEP}Plain"))]
#[case("Plain", "Plain", &format!("{SEP}Plain{SEP}Plain"))]
#[case("Trailing/", "Plain", &format!("{SEP}Trailing{SEP}Plain"))]
#[cfg_attr(not(windows), case("Trailing\\", "Plain", "/Trailing\\/Plain"))]
#[cfg_attr(windows, case("Trailing\\", "Plain", "\\Trailing\\Plain"))]
#[case("Plain", "/Leading", &format!("{SEP}Plain{SEP}Leading"))]
#[cfg_attr(not(windows), case("Plain", "\\Leading", "/Plain/\\Leading"))]
#[cfg_attr(windows, case("Plain", "\\Leading", "\\Plain\\Leading"))]
#[case("Trailing/", "/Leading", &format!("{SEP}Trailing{SEP}{SEP}Leading"))]
#[cfg_attr(not(windows), case("Trailing\\", "\\Leading", "/Trailing\\/\\Leading"))]
#[cfg_attr(windows, case("Trailing\\", "\\Leading", "\\Trailing\\\\Leading"))]
#[case("Plain", "C:\\Abs", &format!("{SEP}Plain{SEP}C:\\Abs"))]
#[case("Trailing/", "C:\\Abs", &format!("{SEP}Trailing{SEP}C:\\Abs"))]
#[cfg_attr(not(windows), case("Trailing\\", "C:\\Abs", "/Trailing\\/C:\\Abs"))]
#[cfg_attr(windows, case("Trailing\\", "C:\\Abs", "\\Trailing\\C:\\Abs"))]
fn remote_appends_name_to_parent(
    #[case] parent: PathBuf,
    #[case] name: &str,
    #[case] expected_str: &str,
) {
    // See `join_raw` tests for more edge cases
    let tendril = Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        "SomeApp",
        name,
        parent.into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();

    let actual = tendril.remote();

    assert_eq!(actual.to_string_lossy(), expected_str);
}

#[rstest]
#[case("", "Plain", &format!("{SEP}G{SEP}Plain"))]
#[case("Plain", "Plain", &format!("{SEP}Plain{SEP}G{SEP}Plain"))]
#[case("Trailing/", "Plain", &format!("{SEP}Trailing{SEP}G{SEP}Plain"))]
#[cfg_attr(not(windows), case("Trailing\\", "Plain", "/Trailing\\/G/Plain"))]
#[cfg_attr(windows, case("Trailing\\", "Plain", "\\Trailing\\G\\Plain"))]
#[case("Plain", "/Leading", &format!("{SEP}Plain{SEP}G{SEP}Leading"))]
#[cfg_attr(not(windows), case("Plain", "\\Leading", "/Plain/G/\\Leading"))]
#[cfg_attr(windows, case("Plain", "\\Leading", "\\Plain\\G\\Leading"))]
#[case("Trailing/", "/Leading", &format!("{SEP}Trailing{SEP}G{SEP}Leading"))]
#[cfg_attr(not(windows), case("Trailing\\", "\\Leading", "/Trailing\\/G/\\Leading"))]
#[cfg_attr(windows, case("Trailing\\", "\\Leading", "\\Trailing\\G\\Leading"))]
#[case(
    "Trailing//",
    "//Both//",
    &format!("{SEP}Trailing{SEP}{SEP}G{SEP}{SEP}Both{SEP}{SEP}"),
)]
#[cfg_attr(not(windows), case(
    "Trailing\\\\",
    "\\\\Both\\\\",
    "/Trailing\\\\/G/\\\\Both\\\\"),
)]
#[cfg_attr(windows, case(
    "Trailing\\\\",
    "\\\\Both\\\\",
    "\\Trailing\\\\G\\\\Both\\\\"),
)]
#[case(
    "Parent///Slashes\\\\.././",
    "Name//.\\Slashes\\\\..",
    &format!("{SEP}Parent{SEP}{SEP}{SEP}Slashes\\\\..{SEP}.{SEP}G{SEP}Name{SEP}{SEP}.\\Slashes\\\\.."),
)]
#[case("Plain", "C:\\Abs", &format!("{SEP}Plain{SEP}G{SEP}C:\\Abs"))]
#[case("Trailing/", "C:\\Abs", &format!("{SEP}Trailing{SEP}G{SEP}C:\\Abs"))]
#[cfg_attr(not(windows), case("Trailing\\", "C:\\Abs", "/Trailing\\/G/C:\\Abs"))]
#[cfg_attr(windows, case("Trailing\\", "C:\\Abs", "\\Trailing\\G\\C:\\Abs"))]
fn local_appends_group_then_name_to_td_repo_replacing_dir_seps_on_windows(
    #[case] td_repo: PathBuf,
    #[case] name: &str,
    #[case] expected_str: &str,
) {
    // See `join_raw` tests for more edge cases
    let tendril = Tendril::new_expose(
        &UniPath::from(td_repo),
        "G",
        name,
        PathBuf::from("Parent").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();

    let actual = tendril.local();

    assert_eq!(actual.to_string_lossy(), expected_str);
}

#[test]
#[serial("mut-env-var-testing")]
fn remote_does_not_resolve_vars_in_name() {
    std::env::set_var("var", "value");

    let tendril = Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        "SomeApp",
        "<var>",
        PathBuf::from("<var>").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();

    let actual = tendril.remote();

    assert_eq!(actual, &PathBuf::from(SEP_STR).join("value").join("<var>"))
}

#[test]
fn remote_preserves_non_utf8_in_parent() {
    let tendril = Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        "SomeApp",
        "misc.txt",
        PathBuf::from(non_utf_8_text()).into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();
    let mut expected_str = std::ffi::OsString::from(SEP_STR);
    expected_str.push(non_utf_8_text());
    expected_str.push(SEP_STR);
    expected_str.push("misc.txt");

    let actual = tendril.remote();

    assert_eq!(actual.as_os_str(), expected_str);
}

#[test]
#[serial("mut-env-var-testing")]
fn local_does_not_resolve_vars_in_name_or_group() {
    std::env::set_var("var", "value");

    let tendril = Tendril::new_expose(
        &UniPath::from(Path::new("<var>")),
        "<var>",
        "<var>",
        PathBuf::from("Parent").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();

    let actual = tendril.local().to_path_buf();

    assert_eq!(
        actual,
        PathBuf::from(SEP_STR).join("value").join("<var>").join("<var>")
    );
}
