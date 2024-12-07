use crate::config::parse_config;
use crate::tests::sample_tendrils::SampleTendrils;
use crate::TendrilMode;

#[test]
fn empty_string_returns_error() {
    let given = "";

    let actual = parse_config(&given);

    assert!(actual.is_err());
    assert!(format!("{:?}", actual).contains("EOF while parsing a value"));
}

#[test]
fn invalid_json_returns_error() {
    let given = "I'm not JSON";

    let actual = parse_config(&given);

    assert!(actual.is_err());
    assert!(format!("{:?}", actual).contains("expected value"));
}

#[test]
fn tendril_json_not_in_array_returns_error() {
    let given = SampleTendrils::tendril_1_json();

    let actual = parse_config(&given);

    assert!(actual.is_err());
    assert!(format!("{:?}", actual).contains("invalid type"));
}

#[test]
fn tendrils_field_is_missing_returns_empty() {
    let given = "{}";

    assert!(parse_config(&given).unwrap().raw_tendrils.is_empty());
}

#[test]
fn tendrils_field_is_null_returns_error() {
    let given = "{\"tendrils\": null}";

    let actual = parse_config(&given);

    assert!(actual.is_err());
    assert!(format!("{:?}", actual).contains("invalid type: null"));
}

#[test]
fn tendrils_field_is_empty_returns_empty() {
    let given = SampleTendrils::build_tendrils_json(&[]);

    assert!(parse_config(&given).unwrap().raw_tendrils.is_empty());
}

#[test]
fn ignores_extra_top_level_fields() {
    let original_json = SampleTendrils::build_tendrils_json(&[
        SampleTendrils::tendril_1_json()
    ]);
    let given = original_json.replacen("{", r#"{"extra-field": true, "#, 1);
    let expected = SampleTendrils::raw_tendrils_1();

    let actual = parse_config(&given).unwrap().raw_tendrils;

    assert_eq!(actual, expected);
}

#[test]
fn json_missing_remotes_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_2_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""remotes": ["some/remote/path/settings2.json"],"#, "");
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given =
        SampleTendrils::build_tendrils_json(&[partial_tendril_json]);

    let actual = parse_config(&given);

    assert!(actual.is_err());
    assert!(format!("{:?}", actual).contains("data did not match any variant of untagged enum OneOrMany"));
}

#[test]
fn json_missing_dir_merge_defaults_to_false() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json =
        original_tendril_json.replace(r#""dir-merge": false,"#, "");
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given =
        SampleTendrils::build_tendrils_json(&[partial_tendril_json]);
    let expected = SampleTendrils::raw_tendrils_1();
    assert_eq!(expected[0].mode, TendrilMode::Overwrite);

    let actual = parse_config(&given).unwrap().raw_tendrils;

    assert_eq!(actual, expected);
}

#[test]
fn json_missing_link_defaults_to_false() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json =
        original_tendril_json.replace(r#""link": false,"#, "");
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given =
        SampleTendrils::build_tendrils_json(&[partial_tendril_json]);
    let expected = SampleTendrils::raw_tendrils_1();
    assert_eq!(expected[0].mode, TendrilMode::Overwrite);

    let actual = parse_config(&given).unwrap().raw_tendrils;

    assert_eq!(actual, expected);
}

#[test]
fn json_missing_profiles_defaults_to_empty() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    // Remove trailing comma and remove profiles field
    let partial_tendril_json = original_tendril_json
        .replace(r#""link": false,"#, r#""link": false"#)
        .replace(r#""profiles": []"#, "");
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given =
        SampleTendrils::build_tendrils_json(&[partial_tendril_json]);
    let expected = SampleTendrils::raw_tendrils_1();
    assert!(expected[0].profiles.is_empty());

    let actual = parse_config(&given).unwrap().raw_tendrils;

    assert_eq!(actual, expected);
}

#[test]
fn json_local_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""settings.json": "#, r#"null: "#);
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given =
        SampleTendrils::build_tendrils_json(&[partial_tendril_json]);

    let actual = parse_config(&given);

    assert!(actual.is_err());
    assert!(format!("{:?}", actual).contains("key must be a string"));
}

#[test]
fn json_remotes_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_2_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""remotes": ["some/remote/path/settings2.json"],"#, r#""remotes": null,"#);
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given =
        SampleTendrils::build_tendrils_json(&[partial_tendril_json]);

    let actual = parse_config(&given);

    assert!(actual.is_err());
    assert!(format!("{:?}", actual).contains("data did not match any variant of untagged enum OneOrMany"));
}

#[test]
fn json_individual_remote_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_2_json();
    let partial_tendril_json = original_tendril_json.replace(
        r#""remotes": ["some/remote/path/settings2.json"],"#,
        r#""remotes": ["some/remote/path/settings2.json", null],"#,
    );
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given =
        SampleTendrils::build_tendrils_json(&[partial_tendril_json]);

    let actual = parse_config(&given);

    assert!(actual.is_err());
    assert!(format!("{:?}", actual).contains("data did not match any variant of untagged enum OneOrMany"));
}

#[test]
fn json_dir_merge_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""dir-merge": false,"#, r#""dir-merge": null,"#);
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given =
        SampleTendrils::build_tendrils_json(&[partial_tendril_json]);

    let actual = parse_config(&given);

    assert!(actual.is_err());
    assert!(format!("{:?}", actual).contains("data did not match any variant of untagged enum OneOrMany"));
}

#[test]
fn json_link_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json =
        original_tendril_json.replace(r#""link": false,"#, r#""link": null,"#);
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given =
        SampleTendrils::build_tendrils_json(&[partial_tendril_json]);

    let actual = parse_config(&given);

    assert!(actual.is_err());
    assert!(format!("{:?}", actual).contains("data did not match any variant of untagged enum OneOrMany"));
}

#[test]
fn json_profiles_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""profiles": []"#, r#""profiles": null"#);
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given =
        SampleTendrils::build_tendrils_json(&[partial_tendril_json]);

    let actual = parse_config(&given);

    assert!(actual.is_err());
    assert!(format!("{:?}", actual).contains("data did not match any variant of untagged enum OneOrMany"));
}

#[test]
fn json_individual_profile_is_null_returns_error() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let partial_tendril_json = original_tendril_json
        .replace(r#""profiles": []"#, r#""profiles": ["mac", null]"#);
    assert_ne!(&original_tendril_json, &partial_tendril_json);

    let given =
        SampleTendrils::build_tendrils_json(&[partial_tendril_json]);

    let actual = parse_config(&given);

    assert!(actual.is_err());
    assert!(format!("{:?}", actual).contains("data did not match any variant of untagged enum OneOrMany"));
}

#[test]
fn single_tendril_in_json_returns_tendril() {
    let given = SampleTendrils::build_tendrils_json(
        &[SampleTendrils::tendril_1_json()],
    );

    let expected = SampleTendrils::raw_tendrils_1();

    let actual = parse_config(&given).unwrap().raw_tendrils;

    assert_eq!(actual, expected);
}

#[test]
fn multiple_tendrils_in_json_returns_tendrils_in_given_order() {
    let given = SampleTendrils::build_tendrils_json(
        &SampleTendrils::all_tendril_jsons(),
    );

    let expected = SampleTendrils::all_tendrils();

    let actual = parse_config(&given).unwrap().raw_tendrils;

    assert_eq!(actual, expected);
}

#[test]
fn ignores_extra_tendril_json_field() {
    let original_tendril_json = SampleTendrils::tendril_1_json();
    let extra_field_tendril_json = original_tendril_json.replace(
        r#""link": false,"#,
        r#""link": false, "extra-field": true,"#,
    );
    assert_ne!(original_tendril_json, extra_field_tendril_json);

    let given = SampleTendrils::build_tendrils_json(
        &[extra_field_tendril_json],
    );

    let expected = SampleTendrils::raw_tendrils_1();
    let actual = parse_config(&given).unwrap().raw_tendrils;

    assert_eq!(actual, expected);
}

#[test]
fn non_list_single_remote_returns_list_of_len_1() {
    let original_tendril_json = SampleTendrils::tendril_2_json();
    let modified_json = original_tendril_json.replace(
        r#""remotes": ["some/remote/path/settings2.json"],"#,
        r#""remotes": "some/remote/path/settings2.json","#,
    );
    assert_ne!(original_tendril_json, modified_json);

    let given = SampleTendrils::build_tendrils_json(&[modified_json]);

    let expected = SampleTendrils::raw_tendrils_2();

    let actual = parse_config(&given).unwrap().raw_tendrils;

    assert_eq!(actual, expected);
    assert_eq!(actual[0].remote, "some/remote/path/settings2.json");
}

#[test]
fn non_list_single_profile_returns_list_of_len_1() {
    let original_tendril_json = SampleTendrils::tendril_2_json();
    let modified_json = original_tendril_json
        .replace(r#""profiles": ["win"]"#, r#""profiles": "win""#);
    assert_ne!(original_tendril_json, modified_json);

    let given = SampleTendrils::build_tendrils_json(&[modified_json]);

    let expected = SampleTendrils::raw_tendrils_2();

    let actual = parse_config(&given).unwrap().raw_tendrils;

    assert_eq!(actual, expected);
    assert_eq!(actual[0].profiles, vec!["win"]);
}
