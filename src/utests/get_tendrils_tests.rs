use crate::{
    get_tendrils,
    GetTendrilsError,
    Tendril,
};
use crate::utests::common::get_disposable_folder;
use crate::utests::sample_tendrils::SampleTendrils;
use serial_test::serial;
use std::matches;
use tempdir::TempDir;

#[test]
#[serial]
fn no_tendrils_json_file_returns_io_error() {
    let temp = TempDir::new_in(get_disposable_folder(), "Empty").unwrap();

    let actual = get_tendrils(&temp.path());

    // TODO: Test for proper IoError variation
    assert!(matches!(actual.unwrap_err(), GetTendrilsError::IoError(_)));
}

#[test]
#[serial]
fn invalid_json_returns_parse_error() {
    let tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "InvalidTendrilsJson"
    ) .unwrap();

    let tendrils_json = &tendrils_folder.path().join("tendrils.json");
    std::fs::File::create(&tendrils_json).unwrap();
    std::fs::write(&tendrils_json, "I'm not JSON").unwrap();

    let actual = get_tendrils(&tendrils_folder.path());

    // TODO: Test for proper ParseError variation
    assert!(matches!(
        actual.unwrap_err(),
        GetTendrilsError::ParseError(_)
    ));
}

#[test]
#[serial]
fn valid_json_returns_tendrils() {
    let tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "ValidJson"
    ).unwrap();
    let json = SampleTendrils::build_tendrils_json(
        &[SampleTendrils::tendril_1_json()].to_vec(),
    );
    let tendrils_json = &tendrils_folder.path().join("tendrils.json");
    std::fs::File::create(&tendrils_json).unwrap();
    std::fs::write(&tendrils_json, &json).unwrap();

    let expected = [SampleTendrils::tendril_1()].to_vec();

    let actual: Vec<Tendril> =
        get_tendrils(&tendrils_folder.path()).unwrap();

    assert_eq!(actual, expected);
}
