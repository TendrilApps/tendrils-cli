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
fn json_missing_group_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json =
        original_tendril_json.replace(r#""group": "MyApp","#, "");
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json.clone()].to_vec(),
    );

    let actual = parse_tendrils(&given);

    assert!(actual.is_err());
}

#[test]
fn json_missing_name_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json =
        original_tendril_json.replace(r#""name": "settings.json","#, "");
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json.clone()].to_vec(),
    );

    let actual = parse_tendrils(&given);

    assert!(actual.is_err());
}

#[test]
fn json_missing_parents_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_2_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""parents": ["some/parent/path"],"#, "");
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json.clone()].to_vec(),
    );

    let actual = parse_tendrils(&given);

    assert!(actual.is_err());
}

#[test]
fn json_missing_dir_merge_defaults_to_false() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""dir-merge": false,"#, "");
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json.clone()].to_vec(),
    );
    let expected = [SampleTendrils::tendril_1()].to_vec();
    assert!(!expected[0].dir_merge);

    let actual = parse_tendrils(&given).unwrap();

    assert!(!actual[0].dir_merge);
}

#[test]
fn json_missing_link_defaults_to_false() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""link": false,"#, "");
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json.clone()].to_vec(),
    );
    let expected = [SampleTendrils::tendril_1()].to_vec();
    assert!(!expected[0].dir_merge);

    let actual = parse_tendrils(&given).unwrap();

    assert!(!actual[0].dir_merge);
}

#[test]
fn json_missing_profiles_defaults_to_empty() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json = original_tendril_json
        // Remove trailing comma
        .replace(r#""link": false,"#, r#""link": false"#)
        // Remove field field
        .replace(r#""profiles": []"#, "");
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json.clone()].to_vec(),
    );
    let expected = [SampleTendrils::tendril_1()].to_vec();
    assert!(expected[0].profiles.is_empty());

    let actual = parse_tendrils(&given).unwrap();

    assert!(actual[0].profiles.is_empty());
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
            SampleTendrils::tendril_4_json(),
            SampleTendrils::tendril_5_json(),
        ].to_vec()
    );

    let expected = [
        SampleTendrils::tendril_1(),
        SampleTendrils::tendril_2(),
        SampleTendrils::tendril_3(),
        SampleTendrils::tendril_4(),
        SampleTendrils::tendril_5(),
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
    assert_ne!(original_tendril_json, extra_field_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[extra_field_tendril_json.clone()].to_vec(),
    );

    let expected = [SampleTendrils::tendril_1()].to_vec();
    let actual = parse_tendrils(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn non_list_single_parent_returns_list_of_len_1() {
    let original_tendril_json = SampleTendrils::tendril_2_json();
    let extra_field_tendril_json = original_tendril_json.replace(
        r#""parents": ["some/parent/path"],"#,
        r#""parents": "some/parent/path","#,
    );
    assert_ne!(original_tendril_json, extra_field_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[extra_field_tendril_json.clone()].to_vec(),
    );

    let expected = [SampleTendrils::tendril_2()].to_vec();

    let actual = parse_tendrils(&given).unwrap();

    assert_eq!(actual, expected);
    assert_eq!(actual[0].parents, vec!["some/parent/path"])
}

#[test]
fn non_list_single_profile_returns_list_of_len_1() {
    let original_tendril_json = SampleTendrils::tendril_2_json();
    let extra_field_tendril_json = original_tendril_json.replace(
        r#""profiles": ["win"]"#,
        r#""profiles": "win""#,
    );
    assert_ne!(original_tendril_json, extra_field_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[extra_field_tendril_json.clone()].to_vec(),
    );

    let expected = [SampleTendrils::tendril_2()].to_vec();

    let actual = parse_tendrils(&given).unwrap();

    assert_eq!(actual, expected);
    assert_eq!(actual[0].profiles, vec!["win"])
}

// TODO: Test when fields are null
