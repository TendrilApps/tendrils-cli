use crate::parse_tendrils;
use crate::libtests::sample_tendrils::SampleTendrils;

#[test]
fn empty_string_returns_error() {
    let given = "";

    assert!(parse_tendrils(&given).is_err());
}

#[test]
fn invalid_json_returns_error() {
    let given = "I'm not JSON";

    assert!(parse_tendrils(&given).is_err());
}

#[test]
fn tendril_json_not_in_array_returns_error() {
    let given = SampleTendrils::tendril_1_json();

    assert!(parse_tendrils(&given).is_err());
}

#[test]
fn json_missing_field_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json =
        original_tendril_json.replace(r#""name": "settings.json","#, "");

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json.clone()].to_vec(),
    );

    let actual = parse_tendrils(&given);

    assert_ne!(&original_tendril_json, &partial_tendril_json);
    assert!(actual.is_err());
}

#[test]
fn empty_json_array_returns_empty() {
    let given = "[]";
    let expected = [].to_vec();

    let actual = parse_tendrils(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn single_tendril_in_json_returns_tendril() {
    let given = SampleTendrils::build_tendrils_json(
        &[SampleTendrils::tendril_1_json()].to_vec(),
    );

    let expected = [SampleTendrils::tendril_1()].to_vec();

    let actual = parse_tendrils(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn multiple_tendrils_in_json_returns_tendrils() {
    let given = SampleTendrils::build_tendrils_json(
        &[
            SampleTendrils::tendril_1_json(),
            SampleTendrils::tendril_2_json(),
            SampleTendrils::tendril_3_json(),
        ].to_vec()
    );

    let expected = [
        SampleTendrils::tendril_1(),
        SampleTendrils::tendril_2(),
        SampleTendrils::tendril_3(),
    ].to_vec();

    let actual = parse_tendrils(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn ignores_extra_json_field_returns_tendril() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let extra_field_tendril_json = original_tendril_json.replace(
        r#""name": "settings.json","#,
        r#""name": "settings.json", "extra field": true,"#,
    );

    let given = SampleTendrils::build_tendrils_json(
        &[extra_field_tendril_json.clone()].to_vec(),
    );

    let expected = [SampleTendrils::tendril_1()].to_vec();
    let actual = parse_tendrils(&given).unwrap();

    assert_ne!(original_tendril_json, extra_field_tendril_json);
    assert_eq!(actual, expected);
}
