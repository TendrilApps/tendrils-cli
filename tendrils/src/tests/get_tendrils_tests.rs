use crate::{
    get_tendrils,
    GetTendrilsError,
    TendrilBundle,
};
use crate::tests::sample_tendrils::SampleTendrils;
use crate::test_utils::get_disposable_dir;
use tempdir::TempDir;

#[test]
fn no_tendrils_json_file_returns_io_not_found_error() {
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();

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
    let temp_td_dir = TempDir::new_in(
        get_disposable_dir(),
        "TendrilsDir"
    ).unwrap();

    let tendrils_json = &temp_td_dir.path().join("tendrils.json");
    std::fs::write(&tendrils_json, "I'm not JSON").unwrap();

    let actual = get_tendrils(&temp_td_dir.path());

    // TODO: Test for proper ParseError variation
    assert!(matches!(
        actual.unwrap_err(),
        GetTendrilsError::ParseError(_)
    ));
}

#[test]
fn valid_json_returns_tendrils_in_same_order_as_file() {
    let temp_td_dir = TempDir::new_in(
        get_disposable_dir(),
        "TendrilsDir"
    ).unwrap();
    let json = SampleTendrils::build_tendrils_json(&vec![
        SampleTendrils::tendril_1_json(),
        SampleTendrils::tendril_4_json(),
        SampleTendrils::tendril_2_json(),
    ]);
    let tendrils_json = &temp_td_dir.path().join("tendrils.json");
    std::fs::write(&tendrils_json, &json).unwrap();

    let expected = vec![
        SampleTendrils::tendril_1(),
        SampleTendrils::tendril_4(),
        SampleTendrils::tendril_2(),
    ];

    let actual: Vec<TendrilBundle> = get_tendrils(&temp_td_dir.path()).unwrap();

    assert_eq!(actual, expected);
}
