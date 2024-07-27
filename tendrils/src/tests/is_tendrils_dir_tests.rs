use crate::{TendrilsActor, TendrilsApi};
use crate::config::parse_config;
use crate::test_utils::get_disposable_dir;
use std::fs::{create_dir_all, write};
use tempdir::TempDir;

#[test]
fn empty_top_level_dir_returns_false() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();

    assert!(!api.is_tendrils_dir(&td_dir.path()));
}

#[test]
fn empty_dot_tendrils_dir_returns_false() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let dot_tendrils_dir = td_dir.path().join(".tendrils");
    create_dir_all(&dot_tendrils_dir).unwrap();

    assert!(!api.is_tendrils_dir(&td_dir.path()));
}

#[test]
fn misc_other_files_only_in_top_level_returns_false() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    write(td_dir.path().join("misc.txt"), "").unwrap();

    assert!(!api.is_tendrils_dir(&td_dir.path()));
}

#[test]
fn valid_tendrils_json_in_top_level_returns_false() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let td_json_file = td_dir.path().join("tendrils.json");
    let json = r#"{"tendrils": []}"#;
    write(td_json_file, json).unwrap();
    assert!(parse_config(json).is_ok());

    assert!(!api.is_tendrils_dir(&td_dir.path()));
}

#[test]
fn tendrils_json_dir_returns_false() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    create_dir_all(td_dir.path().join(".tendrils/tendrils.json")).unwrap();

    assert!(!api.is_tendrils_dir(&td_dir.path()));
}

#[test]
fn empty_tendrils_json_file_returns_true() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let dot_tendrils_dir = td_dir.path().join(".tendrils");
    create_dir_all(&dot_tendrils_dir).unwrap();
    write(dot_tendrils_dir.join("tendrils.json"), "").unwrap();

    assert!(api.is_tendrils_dir(&td_dir.path()));
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
    assert!(parse_config(json).is_err());

    assert!(api.is_tendrils_dir(&td_dir.path()));
}

#[test]
fn valid_tendrils_json_file_returns_true() {
    let api = TendrilsActor {};
    let td_dir = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    let dot_tendrils_dir = td_dir.path().join(".tendrils");
    let td_json_file = dot_tendrils_dir.join("tendrils.json");
    let json = r#"{"tendrils": []}"#;
    create_dir_all(&dot_tendrils_dir).unwrap();
    write(td_json_file, json).unwrap();
    assert!(parse_config(json).is_ok());

    assert!(api.is_tendrils_dir(&td_dir.path()));
}
