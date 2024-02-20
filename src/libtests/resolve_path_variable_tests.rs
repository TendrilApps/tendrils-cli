use crate::{parse_env_variables, resolve_path_variables};
use rstest::rstest;
use serial_test::serial;
use std::path::PathBuf;

#[rstest]
#[case("")]
#[case(" ")]
#[case("\r\n\t")]
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
    let given = "<I_do_not_exist>";
    let expected = PathBuf::from(given);

    let actual = resolve_path_variables(given.to_string());

    assert_eq!(actual, expected);
}

#[rstest]
#[case("<Mut-Testing>")]
#[case("<MUT-TESTING>")]
#[case("<mut-testinG>")]
#[serial("mut-env-var-testing")]
#[cfg_attr(windows, ignore)] // Env vars on Windows are not case sensitive
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
#[case("Path with <mut-testing> spaces", "Path with value spaces")]
#[case("TrailingVar<mut-testing>", "TrailingVarvalue")]
#[case("nested<<mut-testing>>arrows", "nested<value>arrows")]
#[case("offset<<mut-testing>arrows", "offset<valuearrows")]
#[case("offset<mut-testing>>arrows", "offsetvalue>arrows")]
#[case("<mut-testing>/mut-testing", "value/mut-testing")]
#[case("<mut-testing>/duplicate/<mut-testing>", "value/duplicate/value")]
#[case("<mut-testing>/multi/unique/<mut-testing2>", "value/multi/unique/value2")]
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

#[rstest]
#[case("< >")]
#[case("<\n>")]
#[case("<\\>")]
#[case("</>")]
#[case("<|>")]
#[case("<.>")]
#[case("<..>")]
#[case("<var\\name\\that\\is\\a\\path>")]
#[case("<var/name/that/is/a/path>")]
#[case("<var name with spaces>")]
#[serial("mut-env-var-testing")]
fn weird_var_names_still_replace_with_value(
    #[case] var_name: String,
) {
    let given = var_name.clone();
    let expected = PathBuf::from("value");
    let var_no_brkts = &var_name[1..var_name.len() - 1];
    std::env::set_var(var_no_brkts, "value");

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}


#[rstest]
#[case("<var>", "<var>", "<var>")]
#[case("<var>", "var", "var")]
#[case("<var><var><var>", "<var>", "<var><var><var>")]
#[case("<var><var><var>", "var", "varvarvar")]
#[serial("mut-env-var-testing")]
fn value_is_given_var_name_keeps_value(
    #[case] given: String,
    #[case] value: String,
    #[case] expected_str: String
) {
    let expected = PathBuf::from(expected_str);
    std::env::set_var("var", value);

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}

#[rstest]
#[case("<var1><var2>", "var2", "value2", "var2value2")]
#[case("<var2><var1>", "<var2>", "value2", "value2<var2>")]
#[case("<var2><var1>", "var2", "value2", "value2var2")]
#[serial("mut-env-var-testing")]
fn value_is_another_var_name_keeps_value(
    #[case] given: String,
    #[case] var1_value: String,
    #[case] var2_value: String,
    #[case] expected_str: String
) {
    // See also: `value_is_another_var_name_keeps_value_exceptions`
    let expected = PathBuf::from(expected_str);
    std::env::set_var("var1", var1_value);
    std::env::set_var("var2", var2_value);

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}

#[rstest]
#[case("<var1><var2>", "<var2>", "value2", "value2value2")]
#[case("<var1><var2>", "<var2>", "<var1>", "<var1><var1>")]
#[serial("mut-env-var-testing")]
fn value_is_another_var_name_keeps_value_exceptions(
    #[case] given: String,
    #[case] var1_value: String,
    #[case] var2_value: String,
    #[case] expected_str: String
) {
    // These cases are exceptions to `value_is_another_var_name_keeps_value`.
    // Instead of keeping the var name, it's replaced with
    // the new variable's value.
    // This occurs when a variable whose value is another var name
    // occurs before the other variable in the given string.
    // If this case is handled properly in the future,
    // add this test case to the other test.
    let expected = PathBuf::from(expected_str);
    let parsed_vars = parse_env_variables(&var1_value);
    assert!(!parsed_vars.is_empty());
    std::env::set_var("var1", var1_value);
    std::env::set_var("var2", var2_value);

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn var_value_is_non_unicode_returns_lossy_value() {
    let given = "<mut-testing>".to_string();
    let expected = PathBuf::from("fo�o");

    #[cfg(unix)] {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
    
        // Here, the values 0x66 and 0x6f correspond to 'f' and 'o'
        // respectively. The value 0x80 is a lone continuation byte, invalid
        // in a UTF-8 sequence.
        let source = [0x66, 0x6f, 0x80, 0x6f];
        let non_utf8_string = OsStr::from_bytes(&source[..]);
        std::env::set_var("mut-testing", non_utf8_string);
    }
    #[cfg(windows)] {
        use std::ffi::OsString;
        use std::os::windows::prelude::OsStringExt;
    
        // Here the values 0x0066 and 0x006f correspond to 'f' and 'o'
        // respectively. The value 0xD800 is a lone surrogate half, invalid
        // in a UTF-16 sequence.
        let source = [0x0066, 0x006f, 0xD800, 0x006f];
        let os_string = OsString::from_wide(&source[..]);
        let non_utf8_string = os_string.as_os_str();
        std::env::set_var("mut-testing", non_utf8_string);
    }

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}

#[rstest]
#[case("~", "MyHome")]
#[case("~~", "MyHome~")]
#[case("~/", "MyHome/")]
#[case("~\\", "MyHome\\")]
#[case("~/Some/Path", "MyHome/Some/Path")]
#[case("~\\Some\\Path", "MyHome\\Some\\Path")]
#[case("~<var>", "MyHomevalue")]
#[serial("mut-env-var-testing")]
fn leading_tilde_is_replaced_with_home(
    #[case] given: String,
    #[case] expected_str: String
) {
    let expected = PathBuf::from(expected_str);
    std::env::set_var("var", "value");
    std::env::set_var("HOME", "MyHome");

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}

#[rstest]
#[case("")]
#[case(" ")]
#[case("NoTilde")]
#[case("N~onLeadingTilde")]
#[case("NonLeadingTilde~")]
#[serial("mut-env-var-testing")]
fn no_leading_tilde_returns_given(
    #[case] given: String,
) {
    let expected = PathBuf::from(given.clone());
    std::env::set_var("var", "value");
    std::env::set_var("HOME", "MyHome");

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn tilde_value_is_another_var_returns_raw_tilde_value() {
    let home_path = "<var>";
    let expected = PathBuf::from(home_path);
    let parsed_vars = parse_env_variables(home_path);
    assert!(!parsed_vars.is_empty());
    std::env::set_var("HOME", home_path);
    std::env::set_var("var", "value");

    let actual = resolve_path_variables("~".to_string());

    assert_eq!(actual, expected);
}

#[rstest]
#[case("~")]
#[case("~/Some/Path")]
#[case("~\\Some\\Path")]
#[serial("mut-env-var-testing")]
fn tilde_value_doesnt_exist_returns_raw_path(
    #[case] given: String,
) {
    let expected = PathBuf::from(given.clone());
    std::env::remove_var("HOME");

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected)
}

#[test]
#[serial("mut-env-var-testing")]
fn tilde_value_is_non_unicode_returns_lossy_value() {
    let given = "~".to_string();
    let expected = PathBuf::from("fo�o");

    #[cfg(unix)] {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
    
        // Here, the values 0x66 and 0x6f correspond to 'f' and 'o'
        // respectively. The value 0x80 is a lone continuation byte, invalid
        // in a UTF-8 sequence.
        let source = [0x66, 0x6f, 0x80, 0x6f];
        let non_utf8_string = OsStr::from_bytes(&source[..]);
        std::env::set_var("HOME", non_utf8_string);
    }
    #[cfg(windows)] {
        use std::ffi::OsString;
        use std::os::windows::prelude::OsStringExt;
    
        // Here the values 0x0066 and 0x006f correspond to 'f' and 'o'
        // respectively. The value 0xD800 is a lone surrogate half, invalid
        // in a UTF-16 sequence.
        let source = [0x0066, 0x006f, 0xD800, 0x006f];
        let os_string = OsString::from_wide(&source[..]);
        let non_utf8_string = os_string.as_os_str();
        std::env::set_var("HOME", non_utf8_string);
    }

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}
