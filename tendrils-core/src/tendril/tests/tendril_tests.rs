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
#[cfg_attr(not(windows), case("\\"))]
#[cfg_attr(not(windows), case("\\\\"))]
#[cfg_attr(not(windows), case(".\\"))]
#[cfg_attr(not(windows), case("..\\"))]
#[cfg_attr(not(windows), case("\\."))]
#[cfg_attr(not(windows), case("\\.."))]
#[case("some/path")]
#[case("some\\path")]
#[case("/somePath")]
#[case("\\somePath")]
#[case("somePath/")]
#[case("somePath\\")]
#[case("./path")]
#[case(".\\path")]
#[cfg_attr(not(windows), case("..\\path"))]
#[case("Some/./path")]
#[case("Some\\.\\path")]
#[cfg_attr(not(windows), case("Some\\..\\path"))]
#[case("Some/.")]
#[case("Some\\.")]
#[cfg_attr(not(windows), case("Some\\.."))]
#[cfg_attr(not(windows), case(".tendrils\\anything"))]
#[cfg_attr(not(windows), case(".Tendrils\\anything"))]
#[cfg_attr(not(windows), case(".TENDRILS\\anything"))]
#[cfg_attr(not(windows), case(".\\.tendrils"))]
#[cfg_attr(not(windows), case(".\\.tendrils\\anything"))]
#[cfg_attr(not(windows), case("\\.tendrils"))]
#[cfg_attr(not(windows), case("\\.tendrils\\anything"))]
#[cfg_attr(not(windows), case(" .tendrils \\anything"))]
#[cfg_attr(not(windows), case(".tendrils."))]
#[cfg_attr(not(windows), case(".tendrils./anything"))]
#[cfg_attr(not(windows), case(".tendrils.\\anything"))]
#[cfg_attr(not(windows), case("\n .tendrils. \n\\anything"))]
#[case("something/.tendrils")]
#[case("something\\.tendrils")]
#[case("something/tendrils.json")]
#[case("something\\tendrils.json")]
#[case("New\nLine")]
#[case("Carriage\rReturn")]
#[case(".git")]
#[case(".Git")]
#[case(".GIT")]
#[case(".git/anything")]
#[case(".git\\anything")]
#[case(".Git/anything")]
#[case(".Git\\anything")]
#[case(".GIT/anything")]
#[case(".GIT\\anything")]
pub fn valid_locals_and_remotes(#[case] value: &str) {}

#[template]
#[rstest]
#[case("")]
#[case(" ")]
#[case("\n \t")]
#[case("/")]
#[cfg_attr(windows, case("\\"))]
#[case("//")]
#[cfg_attr(windows, case("\\\\"))]
#[case(".")]
#[case("./")]
#[cfg_attr(windows, case(".\\"))]
#[case("../")]
#[cfg_attr(windows, case("..\\"))]
#[case("/.")]
#[cfg_attr(windows, case("\\."))]
#[case("/..")]
#[cfg_attr(windows, case("\\.."))]
#[case("../path")]
#[cfg_attr(windows, case("..\\path"))]
#[case("Some/../path")]
#[cfg_attr(windows, case("Some\\..\\path"))]
#[case("Some/..")]
#[cfg_attr(windows, case("Some\\.."))]
#[case(".tendrils")]
#[case(".Tendrils")]
#[case(".TENDRILS")]
#[case(".tendrils/anything")]
#[cfg_attr(windows, case(".tendrils\\anything"))]
#[case(".Tendrils/anything")]
#[cfg_attr(windows, case(".Tendrils\\anything"))]
#[case(".TENDRILS/anything")]
#[cfg_attr(windows, case(".TENDRILS\\anything"))]
#[case("./.tendrils")]
#[cfg_attr(windows, case(".\\.tendrils"))]
#[case("./.tendrils/anything")]
#[cfg_attr(windows, case(".\\.tendrils\\anything"))]
#[case("/.tendrils")]
#[cfg_attr(windows, case("\\.tendrils"))]
#[case("/.tendrils/anything")]
#[cfg_attr(windows, case("\\.tendrils\\anything"))]
#[case(" .tendrils ")]
#[case(" .tendrils /anything")]
#[cfg_attr(windows, case(" .tendrils \\anything"))]
#[cfg_attr(windows, case(".tendrils."))]
#[cfg_attr(windows, case(".tendrils./anything"))]
#[cfg_attr(windows, case(".tendrils.\\anything"))]
#[case("\n .tendrils \n/anything")]
#[cfg_attr(windows, case("\n .tendrils. \n\\anything"))]
pub fn invalid_locals_but_valid_remotes(#[case] value: &str) {}

#[apply(invalid_locals_but_valid_remotes)]
fn local_is_invalid_returns_invalid_local_error(#[case] local: PathBuf) {
    let actual = Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        local,
        PathBuf::from("SomePath").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap_err();

    assert_eq!(actual, InvalidTendrilError::InvalidLocal);
}

#[apply(valid_locals_and_remotes)]
fn local_is_valid_returns_ok(#[case] local: PathBuf) {
    Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        local,
        PathBuf::from("SomePath").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();
}

#[apply(valid_locals_and_remotes)]
#[case("/")]
#[case("\\")]
#[case("somePath")]
#[case("/some/path/")]
#[case("\\some\\path\\")]
#[case(" / some / path / ")]
#[case(" \\ some \\ path \\ ")]
fn remote_is_valid_returns_ok_or_recursive(#[case] remote: &str) {
    let actual = Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        "SomeLocal".into(),
        PathBuf::from(remote).into(),
        TendrilMode::DirOverwrite,
    );

    assert!(matches!(actual, Ok(_) | Err(InvalidTendrilError::Recursion)));
}

#[apply(invalid_locals_but_valid_remotes)]
fn remote_is_valid_returns_ok_or_recursive_2(#[case] remote: &str) {
    remote_is_valid_returns_ok_or_recursive(remote);
}

#[rstest]
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
#[case(
    "Trailing//",
    "//Both//",
    &format!("{SEP}Trailing{SEP}{SEP}{SEP}{SEP}Both{SEP}{SEP}"),
)]
#[cfg_attr(not(windows), case(
    "Trailing\\\\",
    "\\\\Both\\\\",
    "/Trailing\\\\/\\\\Both\\\\"),
)]
#[cfg_attr(windows, case(
    "Trailing\\\\",
    "\\\\Both\\\\",
    "\\Trailing\\\\\\\\Both\\\\"),
)]
#[case(
    "Parent///Slashes\\\\././",
    "Name//.\\Slashes\\\\.",
    &format!("{SEP}Parent{SEP}{SEP}{SEP}Slashes\\\\.{SEP}.{SEP}Name{SEP}{SEP}.\\Slashes\\\\."),
)]
#[case("Plain", "C:\\Abs", &format!("{SEP}Plain{SEP}C:\\Abs"))]
#[case("Trailing/", "C:\\Abs", &format!("{SEP}Trailing{SEP}C:\\Abs"))]
#[cfg_attr(not(windows), case("Trailing\\", "C:\\Abs", "/Trailing\\/C:\\Abs"))]
#[cfg_attr(windows, case("Trailing\\", "C:\\Abs", "\\Trailing\\C:\\Abs"))]
fn appends_local_to_td_repo_replacing_dir_seps_on_windows(
    #[case] td_repo: PathBuf,
    #[case] local: PathBuf,
    #[case] expected_str: &str,
) {
    // See `join_raw` tests for more edge cases
    let tendril = Tendril::new_expose(
        &UniPath::from(td_repo),
        local,
        PathBuf::from("Remote").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();

    let actual = tendril.local_abs();

    assert_eq!(actual.to_string_lossy(), expected_str);
}

#[test]
#[serial(SERIAL_MUT_ENV_VARS)]
fn local_does_not_resolve_vars() {
    std::env::set_var("var", "value");

    let tendril = Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        "<var>/<var>.txt".into(),
        PathBuf::from("Remote").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();

    let actual = tendril.local_abs();

    assert_eq!(actual, PathBuf::from(format!("/Repo{SEP}<var>/<var>.txt")))
}

#[test]
#[serial(SERIAL_MUT_ENV_VARS)]
fn remote_resolves_any_vars() {
    std::env::set_var("var", "value");

    let tendril = Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        "SomeLocal".into(),
        PathBuf::from("<var>/<var>.txt").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();

    let actual = tendril.remote();

    assert_eq!(
        actual.inner(),
        &PathBuf::from(SEP_STR).join("value").join("value.txt")
    );
}

#[test]
fn local_preserves_non_utf8() {
    let tendril = Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        PathBuf::from(non_utf_8_text()),
        PathBuf::from("Remote").into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();
    let mut expected_str = std::ffi::OsString::from(SEP_STR);
    expected_str.push("Repo");
    expected_str.push(SEP_STR);
    expected_str.push(non_utf_8_text());

    let actual = tendril.local_abs();

    assert_eq!(actual.as_os_str(), expected_str);
}

#[test]
fn remote_preserves_non_utf8() {
    let tendril = Tendril::new_expose(
        &UniPath::from(Path::new("/Repo")),
        "SomeLocal".into(),
        PathBuf::from(non_utf_8_text()).into(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();
    let mut expected_str = std::ffi::OsString::from(SEP_STR);
    expected_str.push(non_utf_8_text());

    let actual = tendril.remote();

    assert_eq!(actual.inner().as_os_str(), expected_str);
}

#[rstest]
#[case("", "/anything")]
#[case("/", "/anything")]
#[case("/path/repo", "/path")]
#[case("/path/repo", "/path/repo")]
#[case("/path/repo", "/path/repo/misc.txt")]
#[case("/path/repo", "/path/repo/nested/misc.txt")]
#[case("/path/repo", "/path/repo/.")]
#[case("/path/repo", "/path/repo/../repo")]
#[case("/path/repo", "/path/repo/nested/..")]
#[case("/path/repo", "./path/repo")]
#[case("/path/repo", "./path/repo/misc.txt")]
#[case("/path/repo/.", "/path/repo")]
#[case("/path/repo/../repo", "/path/repo")]
#[case("/path/repo/nested/..", "/path/repo")]
#[case("/path/./repo", "/path/repo")]
// Not detected properly
// #[case("/path/repo", "../path/repo")]
// #[case("/otherpath/../path/repo", "/path/repo")]
fn recursive_remote_returns_recursion_error(
    #[case] td_repo: PathBuf,
    #[case] remote: PathBuf,
) {
    let actual = Tendril::new(
        UniPath::from(td_repo),
        "SomeLocal".into(),
        UniPath::from(remote),
        TendrilMode::DirOverwrite,
    );

    assert_eq!(actual, Err(InvalidTendrilError::Recursion));
}

#[test]
fn remote_is_sibling_to_given_td_repo_proceeds_normally() {
    let actual = Tendril::new(
        UniPath::from(Path::new("/path/repo")),
        "SomeLocal".into(),
        UniPath::from(Path::new("/path/misc.txt")),
        TendrilMode::DirOverwrite,
    );

    assert!(actual.is_ok());
}
