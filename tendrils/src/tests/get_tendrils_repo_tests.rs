use crate::test_utils::{
    default_repo_path_as_json,
    global_cfg_file,
    Setup,
};
use crate::{get_tendrils_repo, GetTendrilsRepoError, TendrilsActor, TendrilsApi};
use serial_test::serial;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_invalid_default_not_set_returns_given_invalid_err() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let starting_td_repo = setup.td_repo.clone();
    setup.set_home();
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
    let setup = Setup::new();
    let starting_td_repo = setup.td_repo.clone();
    setup.make_global_cfg_file(default_repo_path_as_json("I DON'T EXIST"));
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
    let setup = Setup::new();
    let starting_td_repo = setup.temp_dir.path().join("I don't exist");
    let default_td_repo = setup.td_repo.clone();
    setup.make_td_json_file(&[]);
    setup.make_global_cfg_file(
        default_repo_path_as_json(&default_td_repo.to_string_lossy()),
    );
    assert!(!api.is_tendrils_repo(&starting_td_repo));
    assert!(api.is_tendrils_repo(&default_td_repo));

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
    let setup = Setup::new();
    let starting_td_repo = setup.td_repo.clone();
    setup.make_td_json_file(&[]);
    setup.set_home();
    assert!(!global_cfg_file().exists());
    assert!(api.is_tendrils_repo(&starting_td_repo));

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api).unwrap();

    assert_eq!(actual, starting_td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_valid_default_valid_returns_starting_dir() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let starting_td_repo = setup.td_repo.clone();
    let default_td_repo = setup.temp_dir.path().join("DefaultTendrilsRepo");
    let default_dot_td_dir = default_td_repo.join(".tendrils");
    setup.make_td_json_file(&[]);
    create_dir_all(&default_dot_td_dir).unwrap();
    write(default_dot_td_dir.join("tendrils.json"), "").unwrap();
    setup.make_global_cfg_file(
        default_repo_path_as_json(&default_td_repo.to_string_lossy())
    );
    assert!(api.is_tendrils_repo(&starting_td_repo));
    assert!(api.is_tendrils_repo(&default_td_repo));

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api).unwrap();

    assert_eq!(actual, starting_td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_valid_default_invalid_returns_starting_dir() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let starting_td_repo = setup.td_repo.clone();
    let default_td_repo = setup.temp_dir.path().join("DefaultTendrilsRepo");
    setup.make_td_json_file(&[]);
    setup.make_global_cfg_file(
        default_repo_path_as_json(&default_td_repo.to_string_lossy())
    );
    assert!(api.is_tendrils_repo(&starting_td_repo));
    assert!(!api.is_tendrils_repo(&default_td_repo));

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api).unwrap();

    assert_eq!(actual, starting_td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_none_default_not_set_returns_default_not_set_err() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let starting_td_repo = None;
    setup.set_home();
    assert!(!global_cfg_file().exists());

    let actual = get_tendrils_repo(starting_td_repo, &api);

    assert_eq!(actual, Err(GetTendrilsRepoError::DefaultNotSet));
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_none_default_invalid_returns_default_invalid_err() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let starting_td_repo = None;
    let default_td_repo = PathBuf::from("I DON'T EXIST");
    setup.make_global_cfg_file(
        default_repo_path_as_json(&default_td_repo.to_string_lossy())
    );
    assert!(!api.is_tendrils_repo(&default_td_repo));

    let actual = get_tendrils_repo(starting_td_repo, &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::DefaultInvalid {path: default_td_repo })
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_none_default_valid_returns_default() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let starting_td_repo = None;
    let default_td_repo = setup.td_repo.clone();
    let json_path = default_td_repo.to_string_lossy().replace("\\", "\\\\");
    setup.make_td_json_file(&[]);
    setup.make_global_cfg_file(default_repo_path_as_json(&json_path));
    assert!(api.is_tendrils_repo(&default_td_repo));

    let actual = get_tendrils_repo(starting_td_repo, &api).unwrap();

    assert_eq!(actual, default_td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn leading_tilde_in_given_path_is_resolved() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_td_json_file(&[]);
    setup.set_home();
    let starting_td_repo = PathBuf::from("~/TendrilsRepo");

    let actual = get_tendrils_repo(Some(&starting_td_repo), &api).unwrap();

    assert_eq!(actual, setup.td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn leading_tilde_in_default_path_is_resolved() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_td_json_file(&[]);
    setup.make_global_cfg_file(
        default_repo_path_as_json("~/TendrilsRepo"),
    );

    let actual = get_tendrils_repo(None, &api).unwrap();

    assert_eq!(actual, setup.td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn leading_tilde_in_given_path_is_resolved_in_error_path() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.set_home();
    let starting_td_repo = PathBuf::from("~/TendrilsRepo");

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
    setup.make_global_cfg_file(
        default_repo_path_as_json("~/TendrilsRepo"),
    );

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
    setup.set_home();
    let starting_td_repo = PathBuf::from("Tendrils~Repo");

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
    setup.make_global_cfg_file(
        default_repo_path_as_json("Tendrils~Repo"),
    );

    let actual = get_tendrils_repo(None, &api);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::DefaultInvalid {
            path: PathBuf::from("Tendrils~Repo")
        })
    );
}
