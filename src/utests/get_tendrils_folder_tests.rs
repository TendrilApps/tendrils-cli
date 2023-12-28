use crate::{get_tendrils_folder, is_tendrils_folder};
use crate::utests::common::get_disposable_folder;
use serial_test::serial;
use std::fs::{create_dir_all, File};
use std::env::{remove_var, set_var};
use std::path::PathBuf;
use tempdir::TempDir;

const ENV_NAME: &str = "TENDRILS_FOLDER";

#[test]
#[serial]
fn starting_dir_invalid_env_var_not_set_returns_none() {
    let temp = TempDir::new_in(get_disposable_folder(), "Empty").unwrap();
    remove_var(ENV_NAME);

    let actual = get_tendrils_folder(&temp.path());

    assert!(actual.is_none());
}

#[test]
#[serial]
fn starting_dir_invalid_env_var_invalid_returns_none() {
    let temp = TempDir::new_in(get_disposable_folder(), "Empty").unwrap();
    let env_value = "I DON'T EXIST";
    set_var(ENV_NAME, env_value);

    let actual = get_tendrils_folder(&temp.path());

    assert!(!is_tendrils_folder(&PathBuf::from(env_value)));
    assert!(actual.is_none());
}

#[test]
#[serial]
fn starting_dir_valid_env_var_not_set_returns_starting_dir() {
    let temp = TempDir::new_in(
        get_disposable_folder(),
        "Temp"
    ).unwrap().into_path();
    let starting_tendrils_folder = temp.join("StartingTendrilsFolder");
    create_dir_all(&starting_tendrils_folder).unwrap();
    File::create(starting_tendrils_folder.join("tendrils.json")).unwrap();
    remove_var(ENV_NAME);

    let actual = get_tendrils_folder(&starting_tendrils_folder).unwrap();

    assert_eq!(actual, starting_tendrils_folder);
}

#[test]
#[serial]
fn starting_dir_valid_env_var_valid_returns_starting_dir() {
    let temp = TempDir::new_in(
        get_disposable_folder(),
        "Temp"
    ).unwrap().into_path();
    let starting_tendrils_folder = temp.join("StartingTendrilsFolder");
    let env_var_tendrils_folder = temp.join("EnvVarTendrilsFolder");

    create_dir_all(&starting_tendrils_folder).unwrap();
    create_dir_all(&env_var_tendrils_folder).unwrap();
    File::create(starting_tendrils_folder.join("tendrils.json")).unwrap();
    File::create(env_var_tendrils_folder.join("tendrils.json")).unwrap();
    set_var(ENV_NAME, env_var_tendrils_folder.to_str().unwrap());

    let actual = get_tendrils_folder(&starting_tendrils_folder).unwrap();

    assert!(is_tendrils_folder(&env_var_tendrils_folder));
    assert_eq!(actual, starting_tendrils_folder);
}

#[test]
#[serial]
fn starting_dir_invalid_env_var_valid_returns_env_var() {
    let temp = TempDir::new_in(
        get_disposable_folder(),
        "Temp"
    ).unwrap().into_path();
    let starting_tendrils_folder = temp.join("I don't exist");
    let env_var_tendrils_folder = temp.join("EnvVarTendrilsFolder");

    create_dir_all(&env_var_tendrils_folder).unwrap();
    File::create(env_var_tendrils_folder.join("tendrils.json")).unwrap();
    set_var(ENV_NAME, env_var_tendrils_folder.to_str().unwrap());

    let actual = get_tendrils_folder(&starting_tendrils_folder).unwrap();

    assert!(is_tendrils_folder(&env_var_tendrils_folder));
    assert_eq!(actual, env_var_tendrils_folder);
}
