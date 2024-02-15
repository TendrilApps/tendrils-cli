use crate::resolve_path_variables;
use rstest::rstest;
use serial_test::serial;
use std::path::PathBuf;

#[rstest]
#[case("")]
#[case("some/generic/path")]
#[case("some\\generic\\path")]
#[case("wrong_format_mut-testing")]
#[case("wrong_format_mut-testing>")]
#[case("wrong_format<mut-testing")]
#[case("wrong_format>mut-testing<")]
#[serial("mut-env-var-testing")]
fn no_vars_returns_given_path(#[case] given: String) {
    let expected = PathBuf::from(given.clone());
    std::env::set_var("mut-testing", "value");

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}

#[test]
fn var_doesnt_exist_returns_raw_path() {
    let given = "<I_do_not_exist>".to_string();
    let expected = PathBuf::from(given.clone());

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}

#[rstest]
#[case("<Mut-Testing>")]
#[case("<MUT-TESTING>")]
#[case("<mut-testinG>")]
#[serial("mut-env-var-testing")]
fn wrong_capitalization_of_var_name_returns_raw_path(#[case] given: String) {
    let expected = PathBuf::from(given.clone());
    std::env::set_var("mut-testing", "value");

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}

#[rstest]
#[case("<mut-testing>", "value")]
#[case("some/<mut-testing>/path", "some/value/path")]
#[case("some\\<mut-testing>\\path", "some\\value\\path")]
#[case("sandwiched<mut-testing>Var", "sandwichedvalueVar")]
#[case("<mut-testing>LeadingVar", "valueLeadingVar")]
#[case("TrailingVar<mut-testing>", "TrailingVarvalue")]
#[case("nested<<mut-testing>>arrows", "nested<value>arrows")]
#[case("offset<<mut-testing>arrows", "offset<valuearrows")]
#[case("offset<mut-testing>>arrows", "offsetvalue>arrows")]
#[case("<mut-testing>/multiple/<mut-testing2>", "value/multiple/value2")]
#[serial("mut-env-var-testing")]
fn var_in_path_is_replaced_with_value(
    #[case] given: String,
    #[case] expected_str: String
) {
    let expected = PathBuf::from(expected_str);
    std::env::set_var("mut-testing", "value");
    std::env::set_var("mut-testing2", "value2");

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}

#[test]
#[cfg(not(windows))]
#[serial("mut-env-var-testing")]
fn var_value_is_non_unicode_returns_lossy_value() {
    let given = "<mut-testing>".to_string();

    // Could not get an equivalent test working on Windows.
    // Attempted using OsString::from_wide (from std::os::windows::ffi::OsStringExt)
    // with UTF-16 characters but they were successfully converted to UTF-8 for
    // some reason
    use std::os::unix::ffi::OsStringExt;
    let non_utf8_chars = vec![0x82, 0xAC, 0xFF, 0xFE, 0xFD, 0xFC, 0xD8, 0xDC];
    let non_utf8_string = std::ffi::OsString::from_vec(non_utf8_chars);

    // All characters replaced with the U+FFFD replacement character
    // https://doc.rust-lang.org/std/primitive.char.html#associatedconstant.REPLACEMENT_CHARACTER
    let lossy_string = 
        "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}";
    let expected = PathBuf::from(lossy_string);

    std::env::set_var("mut-testing", non_utf8_string);

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}
