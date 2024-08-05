use crate::{ConfigType, GetConfigError};
use crate::config::{get_global_config, GlobalConfig};
use crate::test_utils::{
    global_cfg_dir,
    get_disposable_dir,
    global_cfg_file,
    set_ra,
};
use serial_test::serial;
use std::env::set_var;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;
use tempdir::TempDir;

const EMPTY_CONFIG: GlobalConfig = GlobalConfig {
    default_repo_path: None,
};

#[test]
#[serial("mut-env-var-testing")]
fn no_config_file_returns_empty_config() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    set_var("HOME", temp.path());
    assert!(!global_cfg_file().exists());

    let actual = get_global_config();

    assert_eq!(
        actual,
        Ok(EMPTY_CONFIG),
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn invalid_json_returns_parse_error() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    set_var("HOME", temp.path());
    create_dir_all(global_cfg_dir()).unwrap();
    write(&global_cfg_file(), "I'm not JSON").unwrap();

    let actual = get_global_config();

    assert_eq!(
        actual,
        Err(GetConfigError::ParseError {
            cfg_type: ConfigType::Global,
            msg: "expected value at line 1 column 1".to_string(),
        }),
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn empty_config_file_returns_parse_error() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    set_var("HOME", temp.path());
    create_dir_all(global_cfg_dir()).unwrap();
    write(&global_cfg_file(), "").unwrap();

    let actual = get_global_config();

    assert_eq!(
        actual,
        Err(GetConfigError::ParseError {
            cfg_type: ConfigType::Global,
            msg: "EOF while parsing a value at line 1 column 0".to_string(),
        }),
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn empty_json_object_returns_empty_config() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    set_var("HOME", temp.path());
    create_dir_all(global_cfg_dir()).unwrap();
    write(&global_cfg_file(), "{}").unwrap();

    let actual = get_global_config();

    assert_eq!(
        actual,
        Ok(EMPTY_CONFIG),
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn no_read_access_to_config_file_returns_io_permission_error() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    set_var("HOME", temp.path());
    let config_file = global_cfg_file();
    create_dir_all(global_cfg_dir()).unwrap();
    write(&config_file, "").unwrap();
    set_ra(&config_file, false);

    let actual = get_global_config();

    set_ra(&config_file, true);
    assert_eq!(
        actual,
        Err(GetConfigError::IoError {
            cfg_type: ConfigType::Global,
            kind: std::io::ErrorKind::PermissionDenied,
        }),
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn valid_json_returns_config_values() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    set_var("HOME", temp.path());
    let json = r#"{"default-repo-path": "Some/Path"}"#;
    create_dir_all(global_cfg_dir()).unwrap();
    write(&global_cfg_file(), json).unwrap();

    let actual = get_global_config();

    assert_eq!(
        actual,
        Ok(GlobalConfig {
            default_repo_path: Some(PathBuf::from("Some/Path")),
        }),
    );
}
