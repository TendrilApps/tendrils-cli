use crate::{
    get_tendrils,
    GetTendrilsError,
    Tendril,
};
use crate::libtests::sample_tendrils::SampleTendrils;
use crate::test_utils::get_disposable_dir;
use tempdir::TempDir;

#[test]
fn no_tendrils_json_file_returns_io_not_found_error() {
    let temp = TempDir::new_in(get_disposable_dir(), "Empty").unwrap();

    let actual = get_tendrils(&temp.path());

    match actual {
        Err(GetTendrilsError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound)
        },
        _ => panic!()
    }
}

#[test]
fn invalid_json_returns_parse_error() {
    let td_dir = TempDir::new_in(
        get_disposable_dir(),
        "InvalidTendrilsJson"
    ).unwrap();

    let tendrils_json = &td_dir.path().join("tendrils.json");
    std::fs::write(&tendrils_json, "I'm not JSON").unwrap();

    let actual = get_tendrils(&td_dir.path());

    // TODO: Test for proper ParseError variation
    assert!(matches!(
        actual.unwrap_err(),
        GetTendrilsError::ParseError(_)
    ));
}

#[test]
fn valid_json_returns_tendrils() {
    let td_dir = TempDir::new_in(
        get_disposable_dir(),
        "ValidJson"
    ).unwrap();
    let json = SampleTendrils::build_tendrils_json(
        &[SampleTendrils::tendril_1_json()].to_vec(),
    );
    let tendrils_json = &td_dir.path().join("tendrils.json");
    std::fs::write(&tendrils_json, &json).unwrap();

    let expected = [SampleTendrils::tendril_1()].to_vec();

    let actual: Vec<Tendril> = get_tendrils(&td_dir.path()).unwrap();

    assert_eq!(actual, expected);
}
