use crate::resolve_path_variables;
use crate::test_utils::get_username_can_panic;
use rstest::rstest;
use serial_test::serial;
use std::path::PathBuf;

#[test]
#[cfg(not(windows))]
fn non_utf_8_path_returns_path_parse_error() {
    // Could not get an equivalent test working on Windows.
    // Attempted using OsString::from_wide (from std::os::windows::ffi::OsStringExt)
    // with UTF-16 characters but they were successfully converted to UTF-8 for
    // some reason
    use std::os::unix::ffi::OsStringExt;
    let non_utf8_chars = vec![
        0xC3, 0x28, 0xA9, 0x29, 0xE2, 0x82, 0xAC, 0xFF, 0xFE, 0xFD, 0xFC,
        0xD8, 0x00, 0xDC, 0x00
    ];

    let non_utf8_string = std::ffi::OsString::from_vec(non_utf8_chars);

    let given = PathBuf::from(non_utf8_string);

    let actual = resolve_path_variables(&given).unwrap_err();

    assert!(matches!(actual, crate::ResolveTendrilError::PathParseError));
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
#[case("<user>", format!("{}", get_username_can_panic()))]
#[case("some/<user>/path", format!("some/{}/path", get_username_can_panic()))]
#[case("some\\<user>\\path", format!("some\\{}\\path", get_username_can_panic()))]
#[case("sandwiched<user>var", format!("sandwiched{}var", get_username_can_panic()))]
#[case("<user>LeadingVar", format!("{}LeadingVar", get_username_can_panic()))]
#[case("TrailingVar<user>", format!("TrailingVar{}", get_username_can_panic()))]
#[case("nested<<user>>arrows", format!("nested<{}>arrows", get_username_can_panic()))]
#[case("<user>/multiple/<user>", format!("{}/multiple/{}", get_username_can_panic(), get_username_can_panic()))]
fn user_var_replaces_with_current_username(
    #[case] given_str: &str,
    #[case] expected_str: String
) {
    let given = PathBuf::from(given_str);
    let expected = PathBuf::from(expected_str);

    let actual = resolve_path_variables(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var")]
fn supported_variable_missing_returns_raw_path() {
    let given = PathBuf::from("<mut-testing>");
    std::env::remove_var("mut-testing");

    let actual = resolve_path_variables(&given).unwrap();

    assert_eq!(actual, given);
}

/// See also `non_utf_8_path_returns_path_parse_error`
#[test]
#[cfg(not(windows))]
#[serial("mut-env-var")]
fn supported_variable_is_non_unicode_returns_raw_path() {
    let given = PathBuf::from("<mut-testing>");

    use std::os::unix::ffi::OsStringExt;
    let non_utf8_chars = vec![
        0xC3, 0x28, 0xA9, 0x29, 0xE2, 0x82, 0xAC, 0xFF, 0xFE, 0xFD, 0xFC,
        0xD8, 0xDC,
    ];
    let non_utf8_string = std::ffi::OsString::from_vec(non_utf8_chars);

    std::env::set_var("mut-testing", non_utf8_string);

    let actual = resolve_path_variables(&given).unwrap();

    assert_eq!(actual, given);
}
