use crate::test_utils::get_disposable_dir;
use crate::tests::sample_tendrils::SampleTendrils;
use crate::{get_config, GetConfigError, TendrilBundle};
use std::fs::{create_dir_all, write};
use tempdir::TempDir;

#[test]
fn no_tendrils_json_file_returns_io_not_found_error() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();

    let actual = get_config(&temp.path());

    assert_eq!(
        actual,
        Err(GetConfigError::IoError { kind: std::io::ErrorKind::NotFound })
    );
}

#[test]
fn invalid_json_returns_parse_error() {
    let temp_td_repo =
        TempDir::new_in(get_disposable_dir(), "TendrilsRepo").unwrap();

    let dot_td_dir = temp_td_repo.path().join(".tendrils");
    let td_json_file = dot_td_dir.join("tendrils.json");
    create_dir_all(dot_td_dir).unwrap();
    write(&td_json_file, "I'm not JSON").unwrap();

    let actual = get_config(&temp_td_repo.path());

    assert_eq!(
        actual,
        Err(GetConfigError::ParseError(
            "expected value at line 1 column 1".to_string()
        ))
    );
}

#[test]
fn valid_json_returns_tendrils_in_same_order_as_file() {
    let temp_td_repo =
        TempDir::new_in(get_disposable_dir(), "TendrilsRepo").unwrap();
    let json = SampleTendrils::build_tendrils_json(&[
        SampleTendrils::tendril_1_json(),
        SampleTendrils::tendril_4_json(),
        SampleTendrils::tendril_2_json(),
    ]);
    let dot_td_dir = temp_td_repo.path().join(".tendrils");
    let td_json_file = dot_td_dir.join("tendrils.json");
    create_dir_all(dot_td_dir).unwrap();
    write(&td_json_file, &json).unwrap();

    let expected = vec![
        SampleTendrils::tendril_1(),
        SampleTendrils::tendril_4(),
        SampleTendrils::tendril_2(),
    ];

    let actual: Vec<TendrilBundle> =
        get_config(&temp_td_repo.path()).unwrap().tendrils;

    assert_eq!(actual, expected);
}
