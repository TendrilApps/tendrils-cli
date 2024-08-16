use crate::path_ext::PathExt;
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

#[test]
#[serial("mut-env-var-testing")]
fn tilde_in_given_path_is_not_resolved() {
    std::env::set_var("HOME", "MyHome");
    let given = "~/<mut-testing>".to_string();
    let expected = PathBuf::from("~/value");
    std::env::set_var("mut-testing", "value");
    // Confirm that the tilde could otherwise be resolved
    assert_ne!(PathBuf::from(&given).resolve_tilde(), PathBuf::from(&given));

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn tilde_in_var_value_is_not_resolved() {
    std::env::set_var("HOME", "MyHome");
    let given = "<mut-testing>".to_string();
    let expected = PathBuf::from("~/value");
    let env_value = "~/value";
    std::env::set_var("mut-testing", &env_value);
    // Confirm that the tilde could otherwise be resolved
    assert_ne!(
        PathBuf::from(&env_value).resolve_tilde(),
        PathBuf::from(&env_value),
    );

    let actual = resolve_path_variables(given);

    assert_eq!(actual, expected);
}
