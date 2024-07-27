use crate::test_utils::get_disposable_dir;
use crate::{get_tendrils_repo, GetTendrilsRepoError, TendrilsActor, TendrilsApi};
use serial_test::serial;
use std::env::{remove_var, set_var};
use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use tempdir::TempDir;

const ENV_NAME: &str = "TENDRILS_REPO";

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_invalid_env_var_not_set_returns_given_invalid_err() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("StartingTendrilsRepo");
    remove_var(ENV_NAME);
    assert!(!api.is_tendrils_repo(&starting_td_repo));

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::GivenInvalid {path: starting_td_repo })
    );
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_invalid_env_var_invalid_returns_given_invalid_err() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("StartingTendrilsRepo");
    let global_td_dir = "I DON'T EXIST";
    set_var(ENV_NAME, global_td_dir);
    assert!(!api.is_tendrils_repo(&starting_td_repo));
    assert!(!api.is_tendrils_repo(&PathBuf::from(global_td_dir)));

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::GivenInvalid {path: starting_td_repo })
    );
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_invalid_env_var_valid_returns_given_invalid_err() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("I don't exist");
    let global_td_dir = temp.path().join("EnvVarTendrilsRepo");
    let global_dot_td_dir = global_td_dir.join(".tendrils");
    let global_td_json_file = global_dot_td_dir.join("tendrils.json");
    create_dir_all(&global_dot_td_dir).unwrap();
    File::create(global_td_json_file).unwrap();
    set_var(ENV_NAME, global_td_dir.to_str().unwrap());
    assert!(!api.is_tendrils_repo(&starting_td_repo));
    assert!(api.is_tendrils_repo(&global_td_dir));

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::GivenInvalid {path: starting_td_repo })
    );
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_valid_env_var_not_set_returns_starting_dir() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("StartingTendrilsRepo");
    let starting_dot_td_dir = starting_td_repo.join(".tendrils");
    create_dir_all(&starting_dot_td_dir).unwrap();
    File::create(starting_dot_td_dir.join("tendrils.json")).unwrap();
    remove_var(ENV_NAME);
    assert!(api.is_tendrils_repo(&starting_td_repo));

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api).unwrap();

    assert_eq!(actual, starting_td_repo);
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_valid_env_var_valid_returns_starting_dir() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("StartingTendrilsRepo");
    let starting_dot_td_dir = starting_td_repo.join(".tendrils");
    let global_td_dir = temp.path().join("EnvVarTendrilsRepo");
    let global_dot_td_dir = global_td_dir.join(".tendrils");
    create_dir_all(&starting_dot_td_dir).unwrap();
    create_dir_all(&global_dot_td_dir).unwrap();
    File::create(starting_dot_td_dir.join("tendrils.json")).unwrap();
    File::create(global_dot_td_dir.join("tendrils.json")).unwrap();
    set_var(ENV_NAME, global_td_dir.to_str().unwrap());
    assert!(api.is_tendrils_repo(&starting_td_repo));
    assert!(api.is_tendrils_repo(&global_td_dir));

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api).unwrap();

    assert_eq!(actual, starting_td_repo);
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_valid_env_var_invalid_returns_starting_dir() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("StartingTendrilsRepo");
    let starting_dot_td_dir = starting_td_repo.join(".tendrils");
    let global_td_dir = temp.path().join("EnvVarTendrilsRepo");
    create_dir_all(&starting_dot_td_dir).unwrap();
    File::create(starting_dot_td_dir.join("tendrils.json")).unwrap();
    set_var(ENV_NAME, global_td_dir.to_str().unwrap());
    assert!(api.is_tendrils_repo(&starting_td_repo));
    assert!(!api.is_tendrils_repo(&global_td_dir));

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api).unwrap();

    assert_eq!(actual, starting_td_repo);
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_none_env_var_not_set_returns_global_not_set_err() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("StartingTendrilsRepo");
    remove_var(ENV_NAME);
    assert!(!api.is_tendrils_repo(&starting_td_repo));

    let actual = get_tendrils_repo(None, &api);

    assert_eq!(actual, Err(GetTendrilsRepoError::GlobalNotSet));
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_none_env_var_invalid_returns_global_invalid_err() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("StartingTendrilsRepo");
    let global_td_dir = "I DON'T EXIST";
    set_var(ENV_NAME, global_td_dir);
    assert!(!api.is_tendrils_repo(&starting_td_repo));
    assert!(!api.is_tendrils_repo(&PathBuf::from(global_td_dir)));

    let actual = get_tendrils_repo(None, &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::GlobalInvalid {path: PathBuf::from(global_td_dir) })
    );
}

#[test]
#[serial("mut-env-var-td-folder")]
fn starting_dir_none_env_var_valid_returns_global() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("I don't exist");
    let global_td_dir = temp.path().join("EnvVarTendrilsRepo");
    let global_dot_td_dir = global_td_dir.join(".tendrils");
    create_dir_all(&global_dot_td_dir).unwrap();
    File::create(global_dot_td_dir.join("tendrils.json")).unwrap();
    set_var(ENV_NAME, global_td_dir.to_str().unwrap());
    assert!(!api.is_tendrils_repo(&starting_td_repo));
    assert!(api.is_tendrils_repo(&global_td_dir));

    let actual = get_tendrils_repo(None, &api).unwrap();

    assert_eq!(actual, global_td_dir);
}
