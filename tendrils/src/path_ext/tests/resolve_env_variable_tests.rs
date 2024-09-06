use crate::path_ext::PathExt;
use crate::test_utils::non_utf_8_text;
use rstest::rstest;
use serial_test::serial;
use std::ffi::OsString;
use std::path::PathBuf;

#[rstest]
#[case("")]
#[case(" ")]
#[case("\r\n\t")]
#[case("<>")]
#[case("Empty<>Brackets")]
#[case("some/generic/path")]
#[case("some\\generic\\path")]
#[case("wrong_format_mut-testing")]
#[case("wrong_format_mut-testing>")]
#[case("wrong_format<mut-testing")]
#[case("wrong_format>mut-testing<")]
#[serial("mut-env-var-testing")]
fn no_vars_returns_given_path(#[case] given: PathBuf) {
    let expected = PathBuf::from(given.clone());
    std::env::set_var("mut-testing", "value");

    let actual = given.resolve_env_variables();

    assert_eq!(actual, expected);
}

#[test]
fn var_doesnt_exist_returns_raw_path() {
    let given = PathBuf::from("<I_do_not_exist>");
    let expected = given.clone();

    let actual = given.resolve_env_variables();

    assert_eq!(actual, expected);
}

#[rstest]
#[case("<Mut-Testing>")]
#[case("<MUT-TESTING>")]
#[case("<mut-testinG>")]
#[serial("mut-env-var-testing")]
#[cfg_attr(windows, ignore)] // Env vars on Windows are not case sensitive
fn wrong_capitalization_of_var_name_returns_raw_path(#[case] given: PathBuf) {
    let expected = given.clone();
    std::env::set_var("mut-testing", "value");

    let actual = given.resolve_env_variables();

    assert_eq!(actual, expected);
}

#[rstest]
#[case("<mut-testing>", "value")]
#[case("some/<mut-testing>/path", "some/value/path")]
#[case("some\\<mut-testing>\\path", "some\\value\\path")]
#[case("/<mut-testing>", "/value")]
#[case("\\<mut-testing>", "\\value")]
#[case("sandwiched<mut-testing>Var", "sandwichedvalueVar")]
#[case("<mut-testing>LeadingVar", "valueLeadingVar")]
#[case("Path with <mut-testing> spaces", "Path with value spaces")]
#[case("TrailingVar<mut-testing>", "TrailingVarvalue")]
#[case("nested<<mut-testing>>arrows", "nested<value>arrows")]
#[case("offset<<mut-testing>arrows", "offset<valuearrows")]
#[case("offset<mut-testing>>arrows", "offsetvalue>arrows")]
#[case("<mut-testing>/<mut-testing>", "value/value")]
#[case("<mut-testing>/mut-testing", "value/mut-testing")]
#[case("<mut-testing>/duplicate/<mut-testing>", "value/duplicate/value")]
#[case(
    "<mut-testing>/multi/unique/<mut-testing2>",
    "value/multi/unique/value2"
)]
#[case("<not-a-var>/<mut-testing>", "<not-a-var>/value")]
// Some Windows path prefixes
#[case("\\\\<mut-testing>\\<mut-testing>", "\\\\value\\value")] // UNC
#[case("\\\\127.0.0.1\\<mut-testing>", "\\\\127.0.0.1\\value")] // UNC
#[case("\\\\.\\<mut-testing>", "\\\\.\\value")] // Device
#[case("\\\\?\\<mut-testing>", "\\\\?\\value")] // Verbatim
#[case("\\\\.\\UNC\\<mut-testing>\\<mut-testing>", "\\\\.\\UNC\\value\\value")]
#[case("\\\\?\\UNC\\<mut-testing>\\<mut-testing>", "\\\\?\\UNC\\value\\value")]
#[serial("mut-env-var-testing")]
fn var_in_path_is_replaced_with_value(
    #[case] given: PathBuf,
    #[case] expected: PathBuf,
) {
    std::env::set_var("mut-testing", "value");
    std::env::set_var("mut-testing2", "value2");

    let actual = given.resolve_env_variables();

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
    let given = PathBuf::from(var_name.clone());
    let expected = PathBuf::from("value");
    let var_no_brkts = &var_name[1..var_name.len() - 1];
    std::env::set_var(var_no_brkts, "value");

    let actual = given.resolve_env_variables();

    assert_eq!(actual, expected);
}

#[rstest]
#[case("<var>", "<var>", "<var>")]
#[case("<var>", "var", "var")]
#[case("<var><var><var>", "<var>", "<var><var><var>")]
#[case("<var><var><var>", "var", "varvarvar")]
#[serial("mut-env-var-testing")]
fn value_is_given_var_name_keeps_value(
    #[case] given: PathBuf,
    #[case] value: String,
    #[case] expected: PathBuf,
) {
    std::env::set_var("var", value);

    let actual = given.resolve_env_variables();

    assert_eq!(actual, expected);
}

#[rstest]
#[case("<var1><var2>", "<var2>", "value2", "<var2>value2")]
#[case("<var2><var1>", "<var2>", "value2", "value2<var2>")]
#[case("<var1><var2>", "<var2>", "<var1>", "<var2><var1>")]
#[case("<var2><var1>", "<var2>", "<var1>", "<var1><var2>")]
#[case("<var1><var2>", "var2", "value2", "var2value2")]
#[case("<var2><var1>", "var2", "value2", "value2var2")]
#[serial("mut-env-var-testing")]
fn value_is_another_var_name_keeps_value(
    #[case] given: PathBuf,
    #[case] var1_value: String,
    #[case] var2_value: String,
    #[case] expected: PathBuf,
) {
    std::env::set_var("var1", var1_value);
    std::env::set_var("var2", var2_value);

    let actual = given.resolve_env_variables();

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn non_utf_8_var_name_is_preserved_if_var_does_not_exist() {
    let mut given_str = OsString::from("Path/With/Non/<");
    given_str.push(non_utf_8_text());
    given_str.push(">/UTF-8");
    let given = PathBuf::from(&given_str);

    let expected = PathBuf::from(&given_str);
    std::env::remove_var(non_utf_8_text());

    let actual = given.resolve_env_variables();

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn non_utf_8_var_name_is_replaced_if_var_exists() {
    let mut given_str = OsString::from("Path/With/Non/<");
    given_str.push(non_utf_8_text());
    given_str.push(">/UTF-8");
    let given = PathBuf::from(&given_str);

    let expected = PathBuf::from("Path/With/Non/value/UTF-8");
    std::env::set_var(&non_utf_8_text(), "value");

    let actual = given.resolve_env_variables();

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn non_utf8_in_var_value_is_preserved() {
    let given = PathBuf::from("<mut-testing>");
    let expected = PathBuf::from(non_utf_8_text());
    std::env::set_var("mut-testing", non_utf_8_text());

    let actual = given.resolve_env_variables();

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn tilde_in_given_path_is_not_resolved() {
    let given = PathBuf::from("~/<mut-testing>");
    let expected = PathBuf::from("~/value");
    std::env::set_var("HOME", "MyHome");
    std::env::set_var("mut-testing", "value");
    // Confirm that the tilde could otherwise be resolved
    assert_ne!(PathBuf::from(&given).resolve_tilde(), PathBuf::from(&given));

    let actual = given.resolve_env_variables();

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn tilde_in_var_value_is_not_resolved() {
    let given = PathBuf::from("<mut-testing>");
    let expected = PathBuf::from("~/value");
    let env_value = "~/value";
    std::env::set_var("HOME", "MyHome");
    std::env::set_var("mut-testing", &env_value);
    // Confirm that the tilde could otherwise be resolved
    assert_ne!(
        PathBuf::from(&env_value).resolve_tilde(),
        PathBuf::from(&env_value),
    );

    let actual = given.resolve_env_variables();

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn dir_seps_are_preserved() {
    let given = PathBuf::from("/path/with\\mixed\\<mut-testing>/dir\\seps");
    let expected_str = "/path/with\\mixed\\value/dir\\seps";
    std::env::set_var("mut-testing", "value");

    let actual = given.resolve_env_variables();

    assert_eq!(actual.to_string_lossy(), expected_str);
}

#[rstest]
#[case("/")]
#[case("\\")]
#[case("/AbsPath")]
#[case("\\AbsPath")]
#[case("C:\\AbsPath")]
#[serial("mut-env-var-testing")]
fn var_value_is_absolute_path_adds_raw_value(#[case] var_value: &str) {
    let given = PathBuf::from("Some/Path/<mut-testing>");
    let expected_str = format!("Some/Path/{var_value}");
    std::env::set_var("mut-testing", var_value);

    let actual = given.resolve_env_variables();

    assert_eq!(actual.to_string_lossy(), expected_str);
}

// TODO: TEst with UNC and verbatim strings
