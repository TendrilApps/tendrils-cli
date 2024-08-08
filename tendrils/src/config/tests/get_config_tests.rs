use crate::{ConfigType, GetConfigError, TendrilBundle};
use crate::config::{Config, get_config};
use crate::test_utils::{get_disposable_dir, Setup};
use crate::tests::sample_tendrils::SampleTendrils;
use std::fs::write;
use tempdir::TempDir;

#[test]
fn no_tendrils_json_file_returns_io_not_found_error() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();

    let actual = get_config(&temp.path());

    assert_eq!(
        actual,
        Err(GetConfigError::IoError {
            cfg_type: ConfigType::Repo,
            kind: std::io::ErrorKind::NotFound,
        })
    );
}

#[test]
fn invalid_json_returns_parse_error() {
    let setup = Setup::new();
    setup.make_dot_td_dir();
    write(&setup.td_json_file, "I'm not JSON").unwrap();

    let actual = get_config(&setup.td_repo);

    assert_eq!(
        actual,
        Err(GetConfigError::ParseError {
            cfg_type: ConfigType::Repo,
            msg: "expected value at line 1 column 1".to_string(),
        }),
    );
}

#[test]
fn empty_config_file_returns_parse_error() {
    let setup = Setup::new();
    setup.make_dot_td_dir();
    write(&setup.td_json_file, "").unwrap();

    let actual = get_config(&setup.td_repo);

    assert_eq!(
        actual,
        Err(GetConfigError::ParseError {
            cfg_type: ConfigType::Repo,
            msg: "EOF while parsing a value at line 1 column 0".to_string(),
        }),
    );
}

#[test]
fn empty_json_object_returns_empty_tendrils_list() {
    let setup = Setup::new();
    setup.make_dot_td_dir();
    write(&setup.td_json_file, "{}").unwrap();

    let actual = get_config(&setup.td_repo);

    assert_eq!(actual, Ok(Config { tendrils: vec![] }));
}

#[test]
fn valid_json_returns_tendrils_in_same_order_as_file() {
    let setup = Setup::new();
    let json = SampleTendrils::build_tendrils_json(&[
        SampleTendrils::tendril_1_json(),
        SampleTendrils::tendril_4_json(),
        SampleTendrils::tendril_2_json(),
    ]);
    setup.make_dot_td_dir();
    write(&setup.td_json_file, &json).unwrap();

    let expected = vec![
        SampleTendrils::tendril_1(),
        SampleTendrils::tendril_4(),
        SampleTendrils::tendril_2(),
    ];

    let actual: Vec<TendrilBundle> =
        get_config(&setup.td_repo).unwrap().tendrils;

    assert_eq!(actual, expected);
}

#[test]
fn config_file_is_unchanged() {
    let setup = Setup::new();
    setup.make_dot_td_dir();
    write(&setup.td_json_file, r#"{"tendrils": []}"#.to_string()).unwrap();

    let _ = get_config(&setup.td_repo).unwrap();

    assert_eq!(setup.td_json_file_contents(), r#"{"tendrils": []}"#);
}
