use crate::path_ext::UniPath;
use crate::test_utils::non_utf_8_text;
use serial_test::serial;
use std::ffi::OsString;
use std::path::PathBuf;
use std::path::MAIN_SEPARATOR as SEP;
use std::path::MAIN_SEPARATOR_STR as SEP_STR;

#[test]
fn replaces_dir_seps_on_init() {
    let given = PathBuf::from("mixed/dir\\seps");
    let expected_str = format!("mixed{SEP}dir{SEP}seps");

    let actual = UniPath::from(given);

    assert_eq!(actual.inner().to_string_lossy(), expected_str);
}

#[test]
#[serial("mut-env-var-testing")]
fn resolves_tilde_on_init() {
    let given = PathBuf::from("~");
    std::env::set_var("HOME", "MyHome");

    let actual = UniPath::from(given);

    assert_eq!(actual.inner().to_string_lossy(), "MyHome");
}

#[test]
#[serial("mut-env-var-testing")]
fn resolves_env_vars_on_init() {
    let given = PathBuf::from("<var>");
    std::env::set_var("var", "value");

    let actual = UniPath::from(given);

    assert_eq!(actual.inner().to_string_lossy(), "value");
}

#[test]
#[serial("mut-env-var-testing")]
fn resolves_tilde_then_vars_then_dir_seps() {
    let given = PathBuf::from("~/<var>\\misc.txt");
    std::env::set_var("HOME", "~/Home/<var>\\");
    std::env::set_var("var", "~/value\\");
    let expected_str = format!(
        "~{SEP}Home{SEP}~{SEP}value{SEP}{SEP}{SEP}~{SEP}value{SEP}{SEP}misc.txt"
    );

    let actual = UniPath::from(given);

    assert_eq!(actual.inner().to_string_lossy(), expected_str);
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

    let mut expected_str = non_utf_8_text();
    expected_str.push(SEP_STR);
    expected_str.push(non_utf_8_text());
    expected_str.push(SEP_STR);
    expected_str.push(non_utf_8_text());

    let actual = UniPath::from(given);

    assert_eq!(actual.inner(), expected_str);
}
