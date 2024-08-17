use crate::path_ext::{contains_env_var, PathExt};
use crate::test_utils::non_utf_8_text;
use rstest::rstest;
use serial_test::serial;
use std::path::PathBuf;

#[rstest]
#[case("~", "MyHome")]
#[case("~/", "MyHome/")]
#[case("~\\", "MyHome\\")]
#[case("~/Some/Path", "MyHome/Some/Path")]
#[case("~\\Some\\Path", "MyHome\\Some\\Path")]
#[case("~/~/Some/Path", "MyHome/~/Some/Path")]
#[case("~\\~\\Some\\Path", "MyHome\\~\\Some\\Path")]
#[serial("mut-env-var-testing")]
fn leading_standalone_tilde_is_replaced_with_home_if_home_exists_regardless_of_fallback_vars(
    #[case] given: PathBuf,
    #[case] expected_str: String,
    #[values(true, false)] homedrive_exists: bool,
    #[values(true, false)] homepath_exists: bool,
) {
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

    let actual = given.resolve_tilde();

    assert_eq!(actual, PathBuf::from(expected_str));
}

#[rstest]
#[case("~", "X:\\MyHomePath")]
#[case("~/", "X:\\MyHomePath/")]
#[case("~\\", "X:\\MyHomePath\\")]
#[case("~/Some/Path", "X:\\MyHomePath/Some/Path")]
#[case("~\\Some\\Path", "X:\\MyHomePath\\Some\\Path")]
#[case("~/~/Some/Path", "X:\\MyHomePath/~/Some/Path")]
#[case("~\\~\\Some\\Path", "X:\\MyHomePath\\~\\Some\\Path")]
#[serial("mut-env-var-testing")]
fn leading_standalone_tilde_is_replaced_with_homedrive_plus_homepath_if_home_doesnt_exist(
    #[case] given: PathBuf,
    #[case] expected_str: String,
) {
    std::env::set_var("HOMEDRIVE", "X:\\");
    std::env::set_var("HOMEPATH", "MyHomePath");
    std::env::remove_var("HOME");

    let actual = given.resolve_tilde();

    assert_eq!(actual, PathBuf::from(expected_str));
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

    let actual = PathBuf::from("~/Some/Path").resolve_tilde();

    assert_eq!(actual, PathBuf::from("~/Some/Path"))
}

#[rstest]
#[case("")]
#[case(" ")]
#[case("~~")]
#[case("/~/~")]
#[case("\\~\\~")]
#[case("NoTilde")]
#[case("~CrowdedTilde")]
#[case("Sandwiched~Tilde")]
#[case("Sandwiched/~/Tilde")]
#[case("Sandwiched\\~\\Tilde")]
#[case("Trailing~")]
#[case("Trailing/~")]
#[case("Trailing\\~")]
#[serial("mut-env-var-testing")]
fn crowded_or_non_leading_tilde_returns_given(#[case] given: PathBuf) {
    let expected = PathBuf::from(given.clone());
    std::env::set_var("var", "value");
    std::env::set_var("HOME", "MyHome");
    std::env::set_var("HOMEDRIVE", "X:\\");
    std::env::set_var("HOMEPATH", "MyHomePath");

    let actual = given.resolve_tilde();

    assert_eq!(actual, expected);
}

#[test]
#[serial("mut-env-var-testing")]
fn var_in_path_is_not_resolved() {
    let given = PathBuf::from("~/Path/With/<var>");
    assert!(contains_env_var(&given));
    std::env::set_var("HOME", "MyHome");
    std::env::set_var("var", "value");

    let actual = given.resolve_tilde();

    assert_eq!(actual, PathBuf::from("MyHome/Path/With/<var>"));
}

#[test]
#[serial("mut-env-var-testing")]
fn tilde_value_is_another_var_returns_raw_tilde_value() {
    let home_value = "<var>";
    let home_path = PathBuf::from(&home_value);
    assert!(contains_env_var(&home_path));
    std::env::set_var("HOME", home_value);
    std::env::set_var("var", "value");

    let actual = PathBuf::from("~").resolve_tilde();

    assert_eq!(actual, home_path);
}

#[test]
#[serial("mut-env-var-testing")]
fn tilde_value_is_another_tilde_returns_raw_tilde_value() {
    std::env::set_var("HOME", "~/Home");

    let actual = PathBuf::from("~").resolve_tilde();

    assert_eq!(actual, PathBuf::from("~/Home"));
}

#[test]
#[serial("mut-env-var-testing")]
fn tilde_value_is_relative_path_returns_raw_tilde_value() {
    std::env::set_var("HOME", "../Home/./..");

    let actual = PathBuf::from("~").resolve_tilde();

    assert_eq!(actual, PathBuf::from("../Home/./.."));
}

#[rstest]
#[case(true)]
#[case(false)]
#[serial("mut-env-var-testing")]
fn non_utf8_in_path_is_preserved(#[case] home_exists: bool) {
    let mut given_str = std::ffi::OsString::from("~/");
    given_str.push(non_utf_8_text());

    if home_exists {
        std::env::set_var("HOME", "MyHome");
    }
    else {
        std::env::remove_var("HOME");
    }
    std::env::set_var("HOMEDRIVE", "X:\\");
    std::env::set_var("HOMEPATH", "MyHomePath");

    let mut expected_str;
    if home_exists {
        expected_str = std::ffi::OsString::from("MyHome/");
    }
    else {
        expected_str = std::ffi::OsString::from("X:\\MyHomePath/");
    }
    expected_str.push(non_utf_8_text());
    let expected = PathBuf::from(expected_str);

    let actual = PathBuf::from(given_str).resolve_tilde();

    assert_eq!(actual, expected);
}


#[rstest]
#[case(true)]
#[case(false)]
#[serial("mut-env-var-testing")]
fn non_utf8_in_tilde_value_is_preserved(#[case] home_exists: bool) {
    let non_utf_8 = non_utf_8_text();
    if home_exists {
        std::env::set_var("HOME", &non_utf_8);
    }
    else {
        std::env::remove_var("HOME");
    }
    std::env::set_var("HOMEDRIVE", &non_utf_8);
    std::env::set_var("HOMEPATH", &non_utf_8);

    let expected;
    if home_exists {
        expected = PathBuf::from(&non_utf_8);
    }
    else {
        let mut double_non_utf_8 = non_utf_8.clone();
        double_non_utf_8.push(non_utf_8.clone());
        expected = PathBuf::from(double_non_utf_8);
    }

    let actual = PathBuf::from("~").resolve_tilde();

    assert_eq!(actual, expected);
}

#[rstest]
#[case("")]
#[case(" ")]
#[case("/")]
#[case("\\")]
#[case("\n")]
#[serial("mut-env-var-testing")]
fn tilde_value_is_misc_returns_raw_tilde_value(#[case] home_path: &str) {
    std::env::set_var("HOME", home_path);

    let actual = PathBuf::from("~").resolve_tilde();

    assert_eq!(actual, PathBuf::from(home_path));
}
