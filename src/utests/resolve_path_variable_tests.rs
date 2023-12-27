use crate::{resolve_path_variables, ResolvePathError};
use crate::utests::common::get_username;
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

#[test]
fn empty_path_returns_empty() {
    let given = PathBuf::from("");
    let expected = given.clone();

    let actual = resolve_path_variables(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn path_without_variables_returns_given_path() {
    let given = PathBuf::from("some/generic/path");
    let expected = given.clone();

    let actual = resolve_path_variables(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn unsupported_var_returns_given_path() {
    let given = PathBuf::from("some/<unsupported>/path");
    let expected = given.clone();

    let actual = resolve_path_variables(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn wrong_capitalized_var_returns_given_path() {
    let given = PathBuf::from("storage/<USER>/my/path");
    let expected = given.clone();

    let actual = resolve_path_variables(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn user_var_replaces_with_current_username() {
    let given = PathBuf::from("storage/<user>/my/path");
    let username = get_username();

    let expected = PathBuf::from(format!("storage/{}/my/path", username));

    let actual = resolve_path_variables(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn sandwiched_var_returns_replaced_path() {
    let given = PathBuf::from("Sandwiched<user>Var");
    let username = get_username();

    let expected = PathBuf::from(format!("Sandwiched{}Var", username));

    let actual = resolve_path_variables(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn leading_var_returns_replaced_path() {
    let given = PathBuf::from("<user>LeadingVar");
    let username = get_username();

    let expected = PathBuf::from(format!("{}LeadingVar", username));

    let actual = resolve_path_variables(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn trailing_var_returns_replaced_path() {
    let given = PathBuf::from("TrailingVar<user>");
    let username = get_username();

    let expected = PathBuf::from(format!("TrailingVar{}", username));

    let actual = resolve_path_variables(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn multiple_var_instances_replaces_all() {
    let given = PathBuf::from("storage/<user>/my/<user>/path");
    let username = get_username();

    let expected = PathBuf::from(format!("storage/{}/my/{}/path", username, username));

    let actual = resolve_path_variables(&given).unwrap();

    assert_eq!(actual, expected);
}
