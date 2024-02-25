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
        &[partial_tendril_json].to_vec(),
    );

    let actual = parse_tendrils(&given);

    assert!(actual.is_err());
}

#[test]
fn json_missing_name_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json =
        original_tendril_json.replace(r#""names": "settings.json","#, "");
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json].to_vec(),
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
        &[partial_tendril_json].to_vec(),
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
        &[partial_tendril_json].to_vec(),
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
        &[partial_tendril_json].to_vec(),
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
        &[partial_tendril_json].to_vec(),
    );
    let expected = [SampleTendrils::tendril_1()].to_vec();
    assert!(expected[0].profiles.is_empty());

    let actual = parse_tendrils(&given).unwrap();

    assert!(actual[0].profiles.is_empty());
}

#[test]
fn json_group_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""group": "MyApp","#, r#""group": null,"#);
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json].to_vec(),
    );

    let actual = parse_tendrils(&given);

    assert!(actual.is_err());
}

#[test]
fn json_names_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""names": "settings.json","#, r#""names": null,"#);
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json].to_vec(),
    );

    let actual = parse_tendrils(&given);

    assert!(actual.is_err());
}

#[test]
fn json_individual_name_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json = original_tendril_json.replace(
        r#""names": "settings.json","#, r#""names": ["settings.json", null],"#
    );
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json].to_vec(),
    );

    let actual = parse_tendrils(&given);

    assert!(actual.is_err());
}

#[test]
fn json_parents_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_2_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""parents": ["some/parent/path"],"#, r#""parents": null,"#);
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json].to_vec(),
    );

    let actual = parse_tendrils(&given);

    assert!(actual.is_err());
}

#[test]
fn json_individual_parent_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_2_json();
    let partial_tendril_json = original_tendril_json.replace(
        r#""parents": ["some/parent/path"],"#,
        r#""parents": ["some/parent/path", null],"#
    );
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json].to_vec(),
    );

    let actual = parse_tendrils(&given);

    assert!(actual.is_err());
}

#[test]
fn json_dir_merge_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""dir-merge": false,"#, r#""dir-merge": null,"#);
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json].to_vec(),
    );

    let actual = parse_tendrils(&given);

    assert!(actual.is_err());
}

#[test]
fn json_link_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""link": false,"#, r#""link": null,"#);
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json].to_vec(),
    );

    let actual = parse_tendrils(&given);

    assert!(actual.is_err());
}

#[test]
fn json_profiles_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""profiles": []"#, r#""profiles": null"#);
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json].to_vec(),
    );

    let actual = parse_tendrils(&given);

    assert!(actual.is_err());
}

#[test]
fn json_individual_profile_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json = original_tendril_json
        .replace( r#""profiles": []"#, r#""profiles": ["mac", null]"#);
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[partial_tendril_json].to_vec(),
    );

    let actual = parse_tendrils(&given);

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
        &SampleTendrils::all_tendril_jsons(),
    );

    let expected = SampleTendrils::all_tendrils();

    let actual = parse_tendrils(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn ignores_extra_json_field_returns_tendril() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let extra_field_tendril_json = original_tendril_json.replace(
        r#""names": "settings.json","#,
        r#""names": "settings.json", "extra field": true,"#,
    );
    assert_ne!(original_tendril_json, extra_field_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[extra_field_tendril_json].to_vec(),
    );

    let expected = [SampleTendrils::tendril_1()].to_vec();
    let actual = parse_tendrils(&given).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn non_list_single_name_returns_list_of_len_1() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    assert!(original_tendril_json.contains(r#""names": "settings.json","#));

    let given = SampleTendrils::build_tendrils_json(
        &[original_tendril_json].to_vec(),
    );

    let expected = [SampleTendrils::tendril_1()].to_vec();

    let actual = parse_tendrils(&given).unwrap();

    assert_eq!(actual, expected);
    assert_eq!(actual[0].names, vec!["settings.json"]);
}

#[test]
fn non_list_single_parent_returns_list_of_len_1() {
    let original_tendril_json = SampleTendrils::tendril_2_json();
    let modified_json = original_tendril_json.replace(
        r#""parents": ["some/parent/path"],"#,
        r#""parents": "some/parent/path","#,
    );
    assert_ne!(original_tendril_json, modified_json);

    let given = SampleTendrils::build_tendrils_json(
        &[modified_json].to_vec(),
    );

    let expected = [SampleTendrils::tendril_2()].to_vec();

    let actual = parse_tendrils(&given).unwrap();

    assert_eq!(actual, expected);
    assert_eq!(actual[0].parents, vec!["some/parent/path"]);
}

#[test]
fn non_list_single_profile_returns_list_of_len_1() {
    let original_tendril_json = SampleTendrils::tendril_2_json();
    let modified_json = original_tendril_json.replace(
        r#""profiles": ["win"]"#,
        r#""profiles": "win""#,
    );
    assert_ne!(original_tendril_json, modified_json);

    let given = SampleTendrils::build_tendrils_json(
        &[modified_json].to_vec(),
    );

    let expected = [SampleTendrils::tendril_2()].to_vec();

    let actual = parse_tendrils(&given).unwrap();

    assert_eq!(actual, expected);
    assert_eq!(actual[0].profiles, vec!["win"]);
}
