use crate::test_utils::{dot_td_dir, get_disposable_dir, repo_path_file};
use crate::{get_tendrils_repo, GetTendrilsRepoError, TendrilsActor, TendrilsApi};
use serial_test::serial;
use std::env::set_var;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;
use tempdir::TempDir;

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_invalid_default_not_set_returns_given_invalid_err() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("StartingTendrilsRepo");
    set_var("HOME", temp.path());
    assert!(!repo_path_file().exists());
    assert!(!api.is_tendrils_repo(&starting_td_repo));

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::GivenInvalid {path: starting_td_repo })
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_invalid_default_invalid_returns_given_invalid_err() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("StartingTendrilsRepo");
    set_var("HOME", temp.path());
    create_dir_all(dot_td_dir()).unwrap();
    write(repo_path_file(), "I DON'T EXIST").unwrap();
    assert!(!api.is_tendrils_repo(&starting_td_repo));

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::GivenInvalid {path: starting_td_repo })
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_invalid_default_valid_returns_given_invalid_err() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("I don't exist");
    let default_td_dir = temp.path().join("EnvVarTendrilsRepo");
    let default_dot_td_dir = default_td_dir.join(".tendrils");
    let default_td_json_file = default_dot_td_dir.join("tendrils.json");
    create_dir_all(&default_dot_td_dir).unwrap();
    write(default_td_json_file, "").unwrap();
    assert!(!api.is_tendrils_repo(&starting_td_repo));
    assert!(api.is_tendrils_repo(&default_td_dir));

    // Create global configs
    set_var("HOME", temp.path());
    create_dir_all(dot_td_dir()).unwrap();
    write(
        repo_path_file(),
        default_td_dir.to_string_lossy().to_string(),
    ).unwrap();

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::GivenInvalid {path: starting_td_repo })
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_valid_default_not_set_returns_starting_dir() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("StartingTendrilsRepo");
    let starting_dot_td_dir = starting_td_repo.join(".tendrils");
    create_dir_all(&starting_dot_td_dir).unwrap();
    write(starting_dot_td_dir.join("tendrils.json"), "").unwrap();
    set_var("HOME", temp.path());
    assert!(!repo_path_file().exists());
    assert!(api.is_tendrils_repo(&starting_td_repo));

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api).unwrap();

    assert_eq!(actual, starting_td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_valid_default_valid_returns_starting_dir() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("StartingTendrilsRepo");
    let starting_dot_td_dir = starting_td_repo.join(".tendrils");
    let default_td_dir = temp.path().join("EnvVarTendrilsRepo");
    let default_dot_td_dir = default_td_dir.join(".tendrils");
    create_dir_all(&starting_dot_td_dir).unwrap();
    create_dir_all(&default_dot_td_dir).unwrap();
    write(starting_dot_td_dir.join("tendrils.json"), "").unwrap();
    write(default_dot_td_dir.join("tendrils.json"), "").unwrap();
    assert!(api.is_tendrils_repo(&starting_td_repo));
    assert!(api.is_tendrils_repo(&default_td_dir));

    // Create global configs
    set_var("HOME", temp.path());
    create_dir_all(dot_td_dir()).unwrap();
    write(
        repo_path_file(),
        default_td_dir.to_string_lossy().to_string(),
    ).unwrap();

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api).unwrap();

    assert_eq!(actual, starting_td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_valid_default_invalid_returns_starting_dir() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("StartingTendrilsRepo");
    let starting_dot_td_dir = starting_td_repo.join(".tendrils");
    let default_td_dir = temp.path().join("EnvVarTendrilsRepo");
    create_dir_all(&starting_dot_td_dir).unwrap();
    write(starting_dot_td_dir.join("tendrils.json"), "").unwrap();
    assert!(api.is_tendrils_repo(&starting_td_repo));
    assert!(!api.is_tendrils_repo(&default_td_dir));

    // Create global configs
    set_var("HOME", temp.path());
    create_dir_all(dot_td_dir()).unwrap();
    write(
        repo_path_file(),
        default_td_dir.to_string_lossy().to_string(),
    ).unwrap();

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api).unwrap();

    assert_eq!(actual, starting_td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_none_default_not_set_returns_default_not_set_err() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("StartingTendrilsRepo");
    set_var("HOME", temp.path());
    assert!(!repo_path_file().exists());
    assert!(!api.is_tendrils_repo(&starting_td_repo));

    let actual = get_tendrils_repo(None, &api);

    assert_eq!(actual, Err(GetTendrilsRepoError::DefaultNotSet));
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_none_default_invalid_returns_default_invalid_err() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("StartingTendrilsRepo");
    let default_td_dir = PathBuf::from("I DON'T EXIST");
    assert!(!api.is_tendrils_repo(&starting_td_repo));
    assert!(!api.is_tendrils_repo(&default_td_dir));

    // Create global configs
    set_var("HOME", temp.path());
    create_dir_all(dot_td_dir()).unwrap();
    write(
        repo_path_file(),
        default_td_dir.to_string_lossy().to_string(),
    ).unwrap();

    let actual = get_tendrils_repo(None, &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::DefaultInvalid {path: PathBuf::from(default_td_dir) })
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_none_default_valid_returns_default() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("I don't exist");
    let default_td_dir = temp.path().join("EnvVarTendrilsRepo");
    let default_dot_td_dir = default_td_dir.join(".tendrils");
    create_dir_all(&default_dot_td_dir).unwrap();
    write(default_dot_td_dir.join("tendrils.json"), "").unwrap();
    assert!(!api.is_tendrils_repo(&starting_td_repo));
    assert!(api.is_tendrils_repo(&default_td_dir));

    // Create global configs
    set_var("HOME", temp.path());
    create_dir_all(dot_td_dir()).unwrap();
    write(
        repo_path_file(),
        default_td_dir.to_string_lossy().to_string(),
    ).unwrap();

    let actual = get_tendrils_repo(None, &api).unwrap();

    assert_eq!(actual, default_td_dir);
}
