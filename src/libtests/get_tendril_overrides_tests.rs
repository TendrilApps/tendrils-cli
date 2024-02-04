use crate::{
    get_tendril_overrides,
    GetTendrilsError,
};
use crate::libtests::sample_tendrils::SampleTendrils;
use crate::test_utils::get_disposable_dir;
use tempdir::TempDir;

#[test]
fn no_tendrils_json_file_returns_empty() {
    let temp_td_dir = TempDir::new_in(
        get_disposable_dir(),
        "TendrilsDir"
    ).unwrap();

    let actual = get_tendril_overrides(&temp_td_dir.path()).unwrap();

    assert!(actual.is_empty())
}

#[test]
fn invalid_json_returns_parse_error() {
    let temp_td_dir = TempDir::new_in(
        get_disposable_dir(),
        "TendrilsDir",
    ).unwrap();

    let tendrils_override_json =
        &temp_td_dir.path().join("tendrils-override.json");
    std::fs::write(&tendrils_override_json, "I'm not JSON").unwrap();

    let actual = get_tendril_overrides(&temp_td_dir.path());

    // TODO: Test for proper ParseError variation
    assert!(matches!(
        actual.unwrap_err(),
        GetTendrilsError::ParseError(_)
    ));
}

#[test]
fn valid_json_returns_tendrils() {
    let temp_td_dir = TempDir::new_in(
        get_disposable_dir(),
        "TendrilsDir"
    ).unwrap();

    let json = SampleTendrils::build_tendrils_json(
        &[SampleTendrils::tendril_1_json()].to_vec(),
    );
    let tendrils_json =
        &temp_td_dir.path().join("tendrils-override.json");
    std::fs::write(&tendrils_json, &json).unwrap();

    let expected = [SampleTendrils::tendril_1()].to_vec();

    let actual = get_tendril_overrides(&temp_td_dir.path()).unwrap();

    assert_eq!(actual, expected);
}
