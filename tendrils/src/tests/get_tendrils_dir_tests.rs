use crate::test_utils::get_disposable_dir;
use crate::{get_tendrils_dir, is_tendrils_dir, GetTendrilsDirError};
use serial_test::serial;
use std::env::{remove_var, set_var};
use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use tempdir::TempDir;

const ENV_NAME: &str = "TENDRILS_FOLDER";

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_invalid_env_var_not_set_returns_given_invalid_err() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_dir = temp.path().join("StartingTendrilsDir");
    remove_var(ENV_NAME);
    assert!(!is_tendrils_dir(&starting_td_dir));

    let actual = get_tendrils_dir(Some(&starting_td_dir));

    assert_eq!(
        actual,
        Err(GetTendrilsDirError::GivenInvalid {path: starting_td_dir })
    );
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_invalid_env_var_invalid_returns_given_invalid_err() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_dir = temp.path().join("StartingTendrilsDir");
    let global_td_dir = "I DON'T EXIST";
    set_var(ENV_NAME, global_td_dir);
    assert!(!is_tendrils_dir(&starting_td_dir));
    assert!(!is_tendrils_dir(&PathBuf::from(global_td_dir)));

    let actual = get_tendrils_dir(Some(&starting_td_dir));

    assert_eq!(
        actual,
        Err(GetTendrilsDirError::GivenInvalid {path: starting_td_dir })
    );
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_invalid_env_var_valid_returns_given_invalid_err() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_dir = temp.path().join("I don't exist");
    let global_td_dir = temp.path().join("EnvVarTendrilsDir");
    create_dir_all(&global_td_dir).unwrap();
    File::create(global_td_dir.join("tendrils.json")).unwrap();
    set_var(ENV_NAME, global_td_dir.to_str().unwrap());
    assert!(!is_tendrils_dir(&starting_td_dir));
    assert!(is_tendrils_dir(&global_td_dir));

    let actual = get_tendrils_dir(Some(&starting_td_dir));

    assert_eq!(
        actual,
        Err(GetTendrilsDirError::GivenInvalid {path: starting_td_dir })
    );
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_valid_env_var_not_set_returns_starting_dir() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_dir = temp.path().join("StartingTendrilsDir");
    create_dir_all(&starting_td_dir).unwrap();
    File::create(starting_td_dir.join("tendrils.json")).unwrap();
    remove_var(ENV_NAME);
    assert!(is_tendrils_dir(&starting_td_dir));

    let actual = get_tendrils_dir(Some(&starting_td_dir)).unwrap();

    assert_eq!(actual, starting_td_dir);
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_valid_env_var_valid_returns_starting_dir() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_dir = temp.path().join("StartingTendrilsDir");
    let global_td_dir = temp.path().join("EnvVarTendrilsDir");
    create_dir_all(&starting_td_dir).unwrap();
    create_dir_all(&global_td_dir).unwrap();
    File::create(starting_td_dir.join("tendrils.json")).unwrap();
    File::create(global_td_dir.join("tendrils.json")).unwrap();
    set_var(ENV_NAME, global_td_dir.to_str().unwrap());
    assert!(is_tendrils_dir(&starting_td_dir));
    assert!(is_tendrils_dir(&global_td_dir));

    let actual = get_tendrils_dir(Some(&starting_td_dir)).unwrap();

    assert_eq!(actual, starting_td_dir);
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_valid_env_var_invalid_returns_starting_dir() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_dir = temp.path().join("StartingTendrilsDir");
    let global_td_dir = temp.path().join("EnvVarTendrilsDir");
    create_dir_all(&starting_td_dir).unwrap();
    File::create(starting_td_dir.join("tendrils.json")).unwrap();
    set_var(ENV_NAME, global_td_dir.to_str().unwrap());
    assert!(is_tendrils_dir(&starting_td_dir));
    assert!(!is_tendrils_dir(&global_td_dir));

    let actual = get_tendrils_dir(Some(&starting_td_dir)).unwrap();

    assert_eq!(actual, starting_td_dir);
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_none_env_var_not_set_returns_global_not_set_err() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_dir = temp.path().join("StartingTendrilsDir");
    remove_var(ENV_NAME);
    assert!(!is_tendrils_dir(&starting_td_dir));

    let actual = get_tendrils_dir(None);

    assert_eq!(actual, Err(GetTendrilsDirError::GlobalNotSet));
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_none_env_var_invalid_returns_global_invalid_err() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_dir = temp.path().join("StartingTendrilsDir");
    let global_td_dir = "I DON'T EXIST";
    set_var(ENV_NAME, global_td_dir);
    assert!(!is_tendrils_dir(&starting_td_dir));
    assert!(!is_tendrils_dir(&PathBuf::from(global_td_dir)));

    let actual = get_tendrils_dir(None);

    assert_eq!(
        actual,
        Err(GetTendrilsDirError::GlobalInvalid {path: PathBuf::from(global_td_dir) })
    );
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_none_env_var_valid_returns_global() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_dir = temp.path().join("I don't exist");
    let global_td_dir = temp.path().join("EnvVarTendrilsDir");
    create_dir_all(&global_td_dir).unwrap();
    File::create(global_td_dir.join("tendrils.json")).unwrap();
    set_var(ENV_NAME, global_td_dir.to_str().unwrap());
    assert!(!is_tendrils_dir(&starting_td_dir));
    assert!(is_tendrils_dir(&global_td_dir));

    let actual = get_tendrils_dir(None).unwrap();

    assert_eq!(actual, global_td_dir);
}
