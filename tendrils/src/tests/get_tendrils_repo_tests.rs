use crate::test_utils::{
    default_repo_path_as_json,
    global_cfg_dir,
    get_disposable_dir,
    global_cfg_file,
    Setup,
};
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
    assert!(!global_cfg_file().exists());
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
    create_dir_all(global_cfg_dir()).unwrap();
    write(
        global_cfg_file(),
        default_repo_path_as_json("I DON'T EXIST"),
    ).unwrap();
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
    let default_td_dir = temp.path().join("DefaultTendrilsRepo");
    let default_dot_td_dir = default_td_dir.join(".tendrils");
    let default_td_json_file = default_dot_td_dir.join("tendrils.json");
    create_dir_all(&default_dot_td_dir).unwrap();
    write(default_td_json_file, "").unwrap();
    assert!(!api.is_tendrils_repo(&starting_td_repo));
    assert!(api.is_tendrils_repo(&default_td_dir));

    // Create global configs
    set_var("HOME", temp.path());
    create_dir_all(global_cfg_dir()).unwrap();
    write(
        global_cfg_file(),
        default_repo_path_as_json(&default_td_dir.to_string_lossy()),
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
    assert!(!global_cfg_file().exists());
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
    let default_td_dir = temp.path().join("DefaultTendrilsRepo");
    let default_dot_td_dir = default_td_dir.join(".tendrils");
    create_dir_all(&starting_dot_td_dir).unwrap();
    create_dir_all(&default_dot_td_dir).unwrap();
    write(starting_dot_td_dir.join("tendrils.json"), "").unwrap();
    write(default_dot_td_dir.join("tendrils.json"), "").unwrap();
    assert!(api.is_tendrils_repo(&starting_td_repo));
    assert!(api.is_tendrils_repo(&default_td_dir));

    // Create global configs
    set_var("HOME", temp.path());
    create_dir_all(global_cfg_dir()).unwrap();
    write(
        global_cfg_file(),
        default_repo_path_as_json(&default_td_dir.to_string_lossy())
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
    let default_td_dir = temp.path().join("DefaultTendrilsRepo");
    create_dir_all(&starting_dot_td_dir).unwrap();
    write(starting_dot_td_dir.join("tendrils.json"), "").unwrap();
    assert!(api.is_tendrils_repo(&starting_td_repo));
    assert!(!api.is_tendrils_repo(&default_td_dir));

    // Create global configs
    set_var("HOME", temp.path());
    create_dir_all(global_cfg_dir()).unwrap();
    write(
        global_cfg_file(),
        default_repo_path_as_json(&default_td_dir.to_string_lossy())
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
    assert!(!global_cfg_file().exists());
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
    create_dir_all(global_cfg_dir()).unwrap();
    write(
        global_cfg_file(),
        default_repo_path_as_json(&default_td_dir.to_string_lossy())
    ).unwrap();

    let actual = get_tendrils_repo(None, &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::DefaultInvalid {path: default_td_dir })
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_none_default_valid_returns_default() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let starting_td_repo = temp.path().join("I don't exist");
    let default_td_dir = temp.path().join("DefaultTendrilsRepo");
    let default_dot_td_dir = default_td_dir.join(".tendrils");
    create_dir_all(&default_dot_td_dir).unwrap();
    write(default_dot_td_dir.join("tendrils.json"), "").unwrap();
    assert!(!api.is_tendrils_repo(&starting_td_repo));
    assert!(api.is_tendrils_repo(&default_td_dir));

    // Create global configs
    let json_path = default_td_dir.to_string_lossy().replace("\\", "\\\\");
    set_var("HOME", temp.path());
    create_dir_all(global_cfg_dir()).unwrap();
    write(
        global_cfg_file(),
        default_repo_path_as_json(&json_path),
    ).unwrap();

    let actual = get_tendrils_repo(None, &api).unwrap();

    assert_eq!(actual, default_td_dir);
}

#[test]
#[serial("mut-env-var-testing")]
fn leading_tilde_in_given_path_is_resolved() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_td_json_file(&[]);
    let starting_td_repo = PathBuf::from("~/TendrilsRepo");
    set_var("HOME", setup.temp_dir.path());

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api).unwrap();

    assert_eq!(actual, setup.td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn leading_tilde_in_default_path_is_resolved() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_td_json_file(&[]);

    // Create global configs
    set_var("HOME", setup.temp_dir.path());
    create_dir_all(global_cfg_dir()).unwrap();
    write(
        global_cfg_file(),
        default_repo_path_as_json("~/TendrilsRepo"),
    ).unwrap();

    let actual = get_tendrils_repo(None, &api).unwrap();

    assert_eq!(actual, setup.td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn leading_tilde_in_given_path_is_resolved_in_error_path() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let starting_td_repo = PathBuf::from("~/TendrilsRepo");
    set_var("HOME", setup.temp_dir.path());

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::GivenInvalid { path: setup.td_repo })
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn leading_tilde_in_default_path_is_resolved_in_error_path() {
    let api = TendrilsActor {};
    let setup = Setup::new();

    // Create global configs
    set_var("HOME", setup.temp_dir.path());
    create_dir_all(global_cfg_dir()).unwrap();
    write(
        global_cfg_file(),
        default_repo_path_as_json("~/TendrilsRepo"),
    ).unwrap();

    let actual = get_tendrils_repo(None, &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::DefaultInvalid { path: setup.td_repo })
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn non_leading_tilde_in_given_path_is_not_resolved() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let starting_td_repo = PathBuf::from("Tendrils~Repo");
    set_var("HOME", setup.temp_dir.path());

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::GivenInvalid {
            path: PathBuf::from("Tendrils~Repo")
        })
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn non_leading_tilde_in_default_path_is_not_resolved() {
    let api = TendrilsActor {};
    let setup = Setup::new();

    // Create global configs
    set_var("HOME", setup.temp_dir.path());
    create_dir_all(global_cfg_dir()).unwrap();
    write(
        global_cfg_file(),
        default_repo_path_as_json("Tendrils~Repo"),
    ).unwrap();

    let actual = get_tendrils_repo(None, &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::DefaultInvalid {
            path: PathBuf::from("Tendrils~Repo")
        })
    );
}
