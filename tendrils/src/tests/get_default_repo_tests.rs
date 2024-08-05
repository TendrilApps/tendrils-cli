use crate::test_utils::{
    default_repo_path_as_json,
    global_cfg_dir,
    get_disposable_dir,
    global_cfg_file,
};
use crate::{ConfigType, GetConfigError, TendrilsActor, TendrilsApi};
use rstest::rstest;
use serial_test::serial;
use std::env::set_var;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;
use tempdir::TempDir;

#[test]
#[serial("mut-env-var-testing")]
fn config_file_does_not_exist_returns_none() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    set_var("HOME", temp.path());
    assert!(!global_cfg_file().exists());

    let actual = api.get_default_repo_path();

    assert_eq!(actual, Ok(None));
}

#[test]
#[serial("mut-env-var-testing")]
fn invalid_json_returns_parse_error() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    set_var("HOME", temp.path());
    create_dir_all(global_cfg_dir()).unwrap();
    write(global_cfg_file(), "").unwrap();

    let actual = api.get_default_repo_path();

    assert_eq!(
        actual,
        Err(GetConfigError::ParseError {
            cfg_type: ConfigType::Global,
            msg: "EOF while parsing a value at line 1 column 0".to_string(),
        }),
    );
}

#[rstest]
#[case("Some/Path", "Some/Path")]
#[case("Some\\\\Path", "Some\\Path")]
#[case("I Do Not Exist", "I Do Not Exist")]
#[case("Multi\\nLine\\nString", "Multi\nLine\nString")]
#[case(" SomePath ", " SomePath ")]
#[case("\\tSomePath\\t", "\tSomePath\t")]
#[case("\\rSomePath\\r", "\rSomePath\r")]
#[case("SomePath\\n", "SomePath\n")]
#[case("\\nSomePath\\n ", "\nSomePath\n ")]
#[case("\\nSome\\nPath\\n ", "\nSome\nPath\n ")]
#[case("\\r\\n \\tSomePath\\r\\n \\t", "\r\n \tSomePath\r\n \t")]
#[serial("mut-env-var-testing")]
fn config_file_exists_returns_unaltered_path_even_if_invalid(
    #[case] field_contents: &str,
    #[case] exp_field_contents: &str
) {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    set_var("HOME", temp.path());
    create_dir_all(global_cfg_dir()).unwrap();
    write(
        global_cfg_file(),
        default_repo_path_as_json(field_contents)
    ).unwrap();

    let actual = api.get_default_repo_path();

    assert_eq!(actual, Ok(Some(PathBuf::from(exp_field_contents))));
}
