use crate::path_ext::UniPath;
use crate::test_utils::non_utf_8_text;
use rstest::rstest;
use serial_test::serial;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::path::MAIN_SEPARATOR as SEP;
use std::path::MAIN_SEPARATOR_STR as SEP_STR;

#[test]
fn replaces_dir_seps_on_init_on_windows() {
    let given = PathBuf::from("/mixed/dir\\seps");
    #[cfg(not(windows))]
    let expected_str = "/mixed/dir\\seps";
    #[cfg(windows)]
    let expected_str = "\\mixed\\dir\\seps";

    let actual = UniPath::from(given);

    assert_eq!(actual.inner().to_string_lossy(), expected_str);
}

#[test]
#[serial("mut-env-var-testing")]
fn resolves_tilde_on_init() {
    let given = PathBuf::from("~");
    std::env::set_var("HOME", "MyHome");
    let expected = format!("{SEP}MyHome");

    let actual = UniPath::from(given);

    assert_eq!(actual.inner().to_string_lossy(), expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn resolves_env_vars_on_init() {
    let given = PathBuf::from("<var>");
    std::env::set_var("var", "value");
    let expected = format!("{SEP}value");

    let actual = UniPath::from(given);

    assert_eq!(actual.inner().to_string_lossy(), expected);
}

#[test]
fn roots_with_dir_sep_on_init() {
    let given = PathBuf::from("some/relative/path");
    let expected = format!("{SEP}some{SEP}relative{SEP}path");

    let actual = UniPath::from(given);

    assert_eq!(actual.inner().to_string_lossy(), expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn resolves_vars_then_tilde_then_dir_seps_then_roots() {
    let given = PathBuf::from("<var>\\misc.txt");
    std::env::set_var("HOME", "~/Home/.//<var>\\");
    std::env::set_var("var", "~/./value\\\\");
    #[cfg(not(windows))]
    let expected_str = "/~/Home/.//<var>\\/./value\\\\\\misc.txt";
    #[cfg(windows)]
    let expected_str =
        "\\~\\Home\\.\\\\<var>\\\\.\\value\\\\\\misc.txt";

    let actual = UniPath::from(given);

    assert_eq!(actual.inner().to_string_lossy(), expected_str);
}

#[rstest]
#[case(".", &format!("{SEP}."))]
#[case("..", &format!("{SEP}.."))]
#[case("/Path", &format!("{SEP}Path"))]
#[cfg_attr(not(windows), case("\\Path", "/\\Path"))]
#[cfg_attr(windows, case("\\Path", "\\Path"))]
#[cfg_attr(not(windows), case("C:\\", "/C:\\"))]
#[cfg_attr(windows, case("C:\\", "C:\\"))]
#[serial("mut-env-var-testing")]
fn resolves_tilde_then_replaces_seps_on_win_then_roots_with_dir_sep(
    #[case] tilde_value: &str,
    #[case] expected_str: &str,
) {
    let given = PathBuf::from("~");
    std::env::set_var("HOME", tilde_value);

    let actual = UniPath::from(given);

    assert_eq!(actual.inner().to_string_lossy(), expected_str);
}

#[rstest]
#[case(".", &format!("{SEP}."))]
#[case("..", &format!("{SEP}.."))]
#[case("/Path", &format!("{SEP}Path"))]
#[cfg_attr(not(windows), case("\\Path", "/\\Path"))]
#[cfg_attr(windows, case("\\Path", "\\Path"))]
#[cfg_attr(not(windows), case("C:\\", "/C:\\"))]
#[cfg_attr(windows, case("C:\\", "C:\\"))]
#[serial("mut-env-var-testing")]
fn resolves_vars_then_replaces_seps_on_win_then_roots_with_dir_sep(
    #[case] var_value: &str,
    #[case] expected_str: &str,
) {
    let given = PathBuf::from("<var>");
    std::env::set_var("var", var_value);

    let actual = UniPath::from(given);

    assert_eq!(actual.inner().to_string_lossy(), expected_str);
}

#[test]
fn new_with_root_prepends_given_root_to_non_rooted_path_and_replaces_dir_seps() {
    let given = PathBuf::from("RelPath");
    #[cfg(not(windows))]
    let expected = "/MyRoot\\/RelPath";
    #[cfg(windows)]
    let expected = "\\MyRoot\\RelPath";

    let actual = UniPath::new_with_root(&given, &Path::new("/MyRoot\\"));

    assert_eq!(actual.inner().to_string_lossy(), expected);
}

#[test]
fn new_with_root_prepends_dir_sep_to_non_rooted_path_if_given_root_is_also_not_rooted() {
    let given = PathBuf::from("RelPath");

    let actual = UniPath::new_with_root(&given, &Path::new("NotARoot"));

    assert_eq!(
        actual.inner().to_string_lossy(),
        format!("{SEP}RelPath")
    );
}

#[test]
fn new_with_root_does_not_prepend_to_a_rooted_path() {
    let given = PathBuf::from("/Rooted/Path");

    let actual = UniPath::new_with_root(&given, &Path::new("/MyRoot\\"));

    assert_eq!(
        actual.inner().to_string_lossy(),
        format!("{SEP}Rooted{SEP}Path")
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn non_utf8_is_preserved() {
    let mut given_str = OsString::from("~/<");
    given_str.push(non_utf_8_text());
    given_str.push(">/");
    given_str.push(non_utf_8_text());
    let given = PathBuf::from(given_str);
    std::env::set_var("HOME", non_utf_8_text());
    std::env::set_var(non_utf_8_text(), non_utf_8_text());

    let mut expected_str = OsString::from(SEP_STR);
    expected_str.push(non_utf_8_text());
    expected_str.push(SEP_STR);
    expected_str.push(non_utf_8_text());
    expected_str.push(SEP_STR);
    expected_str.push(non_utf_8_text());

    let actual = UniPath::from(given);

    assert_eq!(actual.inner().as_os_str(), expected_str);
}
