use crate::{ConfigType, GetConfigError};
use crate::config::{get_global_config, GlobalConfig};
use crate::test_utils::{
    global_cfg_file,
    set_ra,
    Setup,
};
use serial_test::serial;
use std::path::PathBuf;

const EMPTY_CONFIG: GlobalConfig = GlobalConfig {
    default_repo_path: None,
    default_profiles: None,
};

#[test]
#[serial(SERIAL_MUT_ENV_VARS)]
fn no_config_file_returns_empty_config() {
    let setup = Setup::new();
    setup.set_home_dir();
    assert!(!global_cfg_file().exists());

    let actual = get_global_config();

    assert_eq!(
        actual,
        Ok(EMPTY_CONFIG),
    );
}

#[test]
#[serial(SERIAL_MUT_ENV_VARS)]
fn invalid_json_returns_parse_error() {
    let setup = Setup::new();
    setup.make_global_cfg_file("I'm not JSON".to_string());

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
#[serial(SERIAL_MUT_ENV_VARS)]
fn empty_config_file_returns_parse_error() {
    let setup = Setup::new();
    setup.make_global_cfg_file("".to_string());

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
#[serial(SERIAL_MUT_ENV_VARS)]
fn empty_json_object_returns_empty_config() {
    let setup = Setup::new();
    setup.make_global_cfg_file("{}".to_string());

    let actual = get_global_config();

    assert_eq!(
        actual,
        Ok(EMPTY_CONFIG),
    );
}

#[test]
#[serial(SERIAL_MUT_ENV_VARS)]
#[cfg_attr(target_os = "linux", ignore)]
fn no_read_access_to_config_file_returns_io_permission_error() {
    let setup = Setup::new();
    setup.make_global_cfg_file("".to_string());
    let config_file = global_cfg_file();
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
#[serial(SERIAL_MUT_ENV_VARS)]
fn valid_json_returns_config_values() {
    let setup = Setup::new();
    setup.make_global_cfg_file(
        r#"{"default-repo-path": "Some/Path", "default-profiles": ["p1"]}"#.to_string()
    );

    let actual = get_global_config();

    assert_eq!(
        actual,
        Ok(GlobalConfig {
            default_repo_path: Some(PathBuf::from("Some/Path")),
            default_profiles: Some(vec!["p1".to_string()]),
        }),
    );
}

#[test]
#[serial(SERIAL_MUT_ENV_VARS)]
fn config_file_is_unchanged() {
    let setup = Setup::new();
    setup.make_global_cfg_file(
        r#"{"default-repo-path": "Orig text"}"#.to_string()
    );

    let _ = get_global_config().unwrap();

    let global_cfg_file_contents = std::fs::read_to_string(global_cfg_file()).unwrap();
    assert_eq!(global_cfg_file_contents, r#"{"default-repo-path": "Orig text"}"#);
}

