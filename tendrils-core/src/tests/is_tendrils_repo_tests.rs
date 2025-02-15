use crate::{TendrilsActor, TendrilsApi};
use crate::config::parse_config_expose;
use crate::test_utils::{
    get_disposable_dir,
    global_cfg_dir,
    home_dir,
    Setup
};
use serial_test::serial;
use std::fs::{create_dir_all, write};
use tempdir::TempDir;

#[test]
fn empty_top_level_dir_returns_false() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();

    assert!(!api.is_tendrils_repo(&td_dir.path().into()));
}

#[test]
fn empty_dot_tendrils_dir_returns_false() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let dot_tendrils_dir = td_dir.path().join(".tendrils");
    create_dir_all(&dot_tendrils_dir).unwrap();

    assert!(!api.is_tendrils_repo(&td_dir.path().into()));
}

#[test]
fn misc_other_files_only_in_top_level_returns_false() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    write(td_dir.path().join("misc.txt"), "").unwrap();

    assert!(!api.is_tendrils_repo(&td_dir.path().into()));
}

#[test]
fn valid_tendrils_json_in_top_level_returns_false() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let td_json_file = td_dir.path().join("tendrils.json");
    let json = r#"{"tendrils": {}}"#;
    write(td_json_file, json).unwrap();
    assert!(parse_config_expose(json).is_ok());

    assert!(!api.is_tendrils_repo(&td_dir.path().into()));
}

#[test]
fn tendrils_json_dir_returns_false() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    create_dir_all(td_dir.path().join(".tendrils/tendrils.json")).unwrap();

    assert!(!api.is_tendrils_repo(&td_dir.path().into()));
}

#[test]
fn empty_tendrils_json_file_returns_true() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let dot_tendrils_dir = td_dir.path().join(".tendrils");
    create_dir_all(&dot_tendrils_dir).unwrap();
    write(dot_tendrils_dir.join("tendrils.json"), "").unwrap();

    assert!(api.is_tendrils_repo(&td_dir.path().into()));
}

#[test]
fn invalid_tendrils_json_file_returns_true() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let dot_tendrils_dir = td_dir.path().join(".tendrils");
    let td_json_file = dot_tendrils_dir.join("tendrils.json");
    let json = "I'm not json";
    create_dir_all(&dot_tendrils_dir).unwrap();
    write(td_json_file, json).unwrap();
    assert!(parse_config_expose(json).is_err());

    assert!(api.is_tendrils_repo(&td_dir.path().into()));
}

#[test]
fn valid_tendrils_json_file_returns_true() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let dot_tendrils_dir = td_dir.path().join(".tendrils");
    let td_json_file = dot_tendrils_dir.join("tendrils.json");
    let json = r#"{"tendrils": {}}"#;
    create_dir_all(&dot_tendrils_dir).unwrap();
    write(td_json_file, json).unwrap();
    assert!(parse_config_expose(json).is_ok());

    assert!(api.is_tendrils_repo(&td_dir.path().into()));
}

#[test]
#[serial(SERIAL_MUT_ENV_VARS)]
fn home_dir_with_global_cfg_file_and_td_json_returns_true() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_global_cfg_file("".to_string());
    let td_dir = home_dir();
    let dot_tendrils_dir = td_dir.join(".tendrils");
    let td_json_file = dot_tendrils_dir.join("tendrils.json");
    create_dir_all(&dot_tendrils_dir).unwrap();
    write(td_json_file, "").unwrap();

    assert!(api.is_tendrils_repo(&td_dir.into()));
}

#[test]
#[serial(SERIAL_MUT_ENV_VARS)]
fn home_dir_with_global_cfg_file_but_no_td_json_returns_false() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_global_cfg_file("".to_string());
    let td_dir = home_dir();
    assert!(!setup.td_json_file.exists());

    assert!(!api.is_tendrils_repo(&td_dir.into()));
}

#[test]
#[serial(SERIAL_MUT_ENV_VARS)]
fn global_config_dir_can_be_tendrils_folder() {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_global_cfg_file("".to_string());
    let td_dir = global_cfg_dir();
    let dot_tendrils_dir = td_dir.join(".tendrils");
    let td_json_file = dot_tendrils_dir.join("tendrils.json");
    create_dir_all(&dot_tendrils_dir).unwrap();
    write(td_json_file, "").unwrap();

    assert!(api.is_tendrils_repo(&td_dir.into()));
}
