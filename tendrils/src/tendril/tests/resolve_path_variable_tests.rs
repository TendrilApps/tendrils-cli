use crate::tendril::{parse_env_variables, resolve_path_variables};
use crate::test_utils::non_utf_8_text;
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
#[case(
    "<mut-testing>/multi/unique/<mut-testing2>",
    "value/multi/unique/value2"
)]
#[serial("mut-env-var-testing")]
fn var_in_path_is_replaced_with_value(
    #[case] given: String,
    #[case] expected_str: String,
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
fn weird_var_names_still_replace_with_value(#[case] var_name: String) {
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
    #[case] expected_str: String,
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
    #[case] expected_str: String,
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
    #[case] expected_str: String,
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
    std::env::set_var("mut-testing", non_utf_8_text());

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
fn leading_tilde_is_replaced_with_home_if_home_exists_regardless_of_fallback_vars(
    #[case] given: String,
    #[case] expected_str: String,
    #[values(true, false)] homedrive_exists: bool,
    #[values(true, false)] homepath_exists: bool,
) {
    let expected = PathBuf::from(expected_str);
    std::env::set_var("var", "value");
    std::env::set_var("HOME", "MyHome");
    if homedrive_exists {
        std::env::set_var("HOMEDRIVE", "X:\\");
    }
    else {
        std::env::remove_var("HOMEDRIVE");
    }
    if homepath_exists {
        std::env::set_var("HOMEPATH", "MyHomePath");
    }
    else {
        std::env::remove_var("HOMEPATH");
    }

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}

#[rstest]
#[case("~", "X:\\MyHomePath")]
#[case("~~", "X:\\MyHomePath~")]
#[case("~/", "X:\\MyHomePath/")]
#[case("~\\", "X:\\MyHomePath\\")]
#[case("~/Some/Path", "X:\\MyHomePath/Some/Path")]
#[case("~\\Some\\Path", "X:\\MyHomePath\\Some\\Path")]
#[case("~<var>", "X:\\MyHomePathvalue")]
#[serial("mut-env-var-testing")]
fn leading_tilde_is_replaced_with_homedrive_plus_homepath_if_home_doesnt_exist(
    #[case] given: String,
    #[case] expected_str: String,
) {
    let expected = PathBuf::from(expected_str);
    std::env::set_var("var", "value");
    std::env::set_var("HOMEDRIVE", "X:\\");
    std::env::set_var("HOMEPATH", "MyHomePath");
    std::env::remove_var("HOME");

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}

#[rstest]
#[case(false, false)]
#[case(true, false)]
#[case(false, true)]
#[serial("mut-env-var-testing")]
fn leading_tilde_returns_given_if_home_and_either_homedrive_or_homepath_dont_exist(
    #[case] homedrive_exists: bool,
    #[case] homepath_exists: bool,
) {
    std::env::remove_var("HOME");
    if homedrive_exists {
        std::env::set_var("HOMEDRIVE", "X:\\");
    }
    else {
        std::env::remove_var("HOMEDRIVE");
    }
    if homepath_exists {
        std::env::set_var("HOMEPATH", "MyHomePath");
    }
    else {
        std::env::remove_var("HOMEPATH");
    }

    let actual = resolve_path_variables("~/Some/Path".to_string());

    assert_eq!(actual, PathBuf::from("~/Some/Path"))
}

#[rstest]
#[case("")]
#[case(" ")]
#[case("NoTilde")]
#[case("N~onLeadingTilde")]
#[case("NonLeadingTilde~")]
#[serial("mut-env-var-testing")]
fn no_leading_tilde_returns_given(#[case] given: String) {
    let expected = PathBuf::from(given.clone());
    std::env::set_var("var", "value");
    std::env::set_var("HOME", "MyHome");
    std::env::set_var("HOMEDRIVE", "X:\\");
    std::env::set_var("HOMEPATH", "MyHomePath");

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
#[case(true, "fo�o")]
#[case(false, "fo�ofo�o")]
#[serial("mut-env-var-testing")]
fn tilde_value_is_non_unicode_returns_lossy_value(
    #[case] home_exists: bool,
    #[case] expected_str: &str,
) {
    let given = "~".to_string();
    let expected = PathBuf::from(expected_str);

    if home_exists {
        std::env::set_var("HOME", non_utf_8_text());
    }
    else {
        std::env::remove_var("HOME");
    }
    std::env::set_var("HOMEDRIVE", non_utf_8_text());
    std::env::set_var("HOMEPATH", non_utf_8_text());

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}
