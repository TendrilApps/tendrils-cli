use crate::{resolve_path_variables, ResolvePathError};
use crate::utests::common::get_username;
use rstest::rstest;
use std::path::PathBuf;

#[test]
#[cfg(unix)]
// Could not get an equivalent test working on Windows.
// Attempted using OsString::from_wide (from std::os::windows::ffi::OsStringExt)
// with UTF-16 characters but they were successfully converted to UTF-8 for
// some reason
fn non_utf_8_path_returns_path_parse_error() {
    use std::os::unix::ffi::OsStringExt;
    let non_utf8_chars = vec![
        0xC3, 0x28, 0xA9, 0x29, 0xE2, 0x82, 0xAC, 0xFF, 0xFE, 0xFD, 0xFC,
        0xD8, 0x00, 0xDC, 0x00
    ];

    let non_utf8_string = std::ffi::OsString::from_vec(non_utf8_chars);

    let given = PathBuf::from(non_utf8_string);

    let actual = resolve_path_variables(&given).unwrap_err();

    assert!(matches!(actual, ResolvePathError::PathParseError));
}

#[rstest]
#[case("")]
#[case("some/generic/path")]
#[case("some\\generic\\path")]
#[case("<unsupported>")]
#[case("some/<unsupported>/path")]
#[case("some\\<unsupported>\\path")]
#[case("wrong_format_user")]
#[case("wrong_format_<user")]
#[case("wrong_format_user>")]
#[case("wrong_capitalization_<USER>")]
fn no_supported_variables_returns_given_path(#[case] given_str: &str) {
    let given = PathBuf::from(given_str);
    let expected = given.clone();

    let actual = resolve_path_variables(&given).unwrap();

    assert_eq!(actual, expected);
}

#[rstest]
#[case("<user>", format!("{}", get_username()))]
#[case("some/<user>/path", format!("some/{}/path", get_username()))]
#[case("some\\<user>\\path", format!("some\\{}\\path", get_username()))]
#[case("sandwiched<user>var", format!("sandwiched{}var", get_username()))]
#[case("<user>LeadingVar", format!("{}LeadingVar", get_username()))]
#[case("TrailingVar<user>", format!("TrailingVar{}", get_username()))]
#[case("nested<<user>>arrows", format!("nested<{}>arrows", get_username()))]
#[case("<user>/multiple/<user>", format!("{}/multiple/{}", get_username(), get_username()))]
fn user_var_replaces_with_current_username(
    #[case] given_str: &str,
    #[case] expected_str: String
) {
    let given = PathBuf::from(given_str);
    let expected = PathBuf::from(expected_str);

    let actual = resolve_path_variables(&given).unwrap();

    assert_eq!(actual, expected);
}
