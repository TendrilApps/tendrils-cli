use crate::config::{Config, parse_config};
use crate::test_utils::Setup;
use crate::{
    get_config,
    InitError,
    TendrilBundle,
    TendrilsActor,
    TendrilsApi,
};
use rstest::rstest;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;

#[rstest]
#[case(true)]
#[case(false)]
fn creates_dot_tendrils_dir_and_contents_in_empty_dir(#[case] force: bool) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_td_dir();
    let expected_t1 = TendrilBundle {
        group: "SomeApp".to_string(),
        names: vec!["SomeFile.ext".to_string()],
        parents: vec!["path/to/containing/folder".to_string()],
        dir_merge: false,
        link: false,
        profiles: vec![],
    };
    let expected_t2 = TendrilBundle {
        group: "SomeApp2".to_string(),
        names: vec!["SomeFile2.ext".to_string(), "SomeFolder3".to_string()],
        parents: vec![
            "path/to/containing/folder2".to_string(),
            "path/to/containing/folder3".to_string(),
            "path/to/containing/folder4".to_string(),
        ],
        dir_merge: false,
        link: true,
        profiles: vec!["home".to_string(), "work".to_string()],
    };
    let expected_tendrils = vec![expected_t1, expected_t2];
    let expected = Config { tendrils: expected_tendrils };

    let actual = api.init_tendrils_dir(&setup.td_dir, force);

    assert_eq!(actual, Ok(()));
    assert_eq!(setup.td_json_file_contents(), crate::INIT_TD_TENDRILS_JSON);
    assert!(api.is_tendrils_dir(&setup.td_dir));
    assert_eq!(get_config(&setup.td_dir).unwrap(), expected);
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_doesnt_exist_returns_io_error_not_found(#[case] force: bool) {
    let api = TendrilsActor {};
    let dir = PathBuf::from("I do not exist");

    let actual = api.init_tendrils_dir(&dir, force);

    assert!(!dir.join("tendrils.json").exists());
    assert_eq!(
        actual,
        Err(InitError::IoError { kind: std::io::ErrorKind::NotFound })
    );
    assert!(!api.is_tendrils_dir(&dir))
}

#[rstest]
#[rstest]
#[case(true)]
#[case(false)]
fn dir_contains_another_misc_file_returns_not_empty_error_unless_forced(
    #[case] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_td_dir();
    let misc_file = setup.td_dir.join("misc.txt");
    write(&misc_file, "Misc file contents").unwrap();

    let actual = api.init_tendrils_dir(&setup.td_dir, force);

    assert_eq!(read_to_string(misc_file).unwrap(), "Misc file contents");
    if force {
        assert_eq!(actual, Ok(()));
        assert_eq!(setup.td_json_file_contents(), crate::INIT_TD_TENDRILS_JSON);
        assert!(api.is_tendrils_dir(&setup.td_dir))
    }
    else {
        assert_eq!(actual, Err(InitError::NotEmpty));
        assert!(!api.is_tendrils_dir(&setup.td_dir))
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_contains_another_misc_dir_returns_not_empty_error_unless_forced(
    #[case] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let misc_dir = setup.td_dir.join("misc");
    let misc_nested = misc_dir.join("nested.txt");
    create_dir_all(&misc_dir).unwrap();
    write(&misc_nested, "Nested file contents").unwrap();

    let actual = api.init_tendrils_dir(&setup.td_dir, force);

    assert_eq!(read_to_string(misc_nested).unwrap(), "Nested file contents");
    if force {
        assert_eq!(actual, Ok(()));
        assert_eq!(setup.td_json_file_contents(), crate::INIT_TD_TENDRILS_JSON);
        assert!(api.is_tendrils_dir(&setup.td_dir))
    }
    else {
        assert_eq!(actual, Err(InitError::NotEmpty));
        assert!(!api.is_tendrils_dir(&setup.td_dir))
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_contains_empty_dot_tendrils_dir_returns_not_empty_error_unless_forced(
    #[case] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_dot_td_dir();

    let actual = api.init_tendrils_dir(&setup.td_dir, force);

    if force {
        assert_eq!(actual, Ok(()));
        assert_eq!(setup.td_json_file_contents(), crate::INIT_TD_TENDRILS_JSON);
        assert!(api.is_tendrils_dir(&setup.td_dir))
    }
    else {
        assert_eq!(actual, Err(InitError::NotEmpty));
        assert!(!api.is_tendrils_dir(&setup.td_dir))
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_contains_non_empty_dot_tendrils_dir_returns_not_empty_error_unless_forced(
    #[case] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let misc_nested = setup.dot_td_dir.join("nested.txt");
    setup.make_dot_td_dir();
    write(&misc_nested, "Nested file contents").unwrap();

    let actual = api.init_tendrils_dir(&setup.td_dir, force);

    assert_eq!(read_to_string(misc_nested).unwrap(), "Nested file contents");
    if force {
        assert_eq!(actual, Ok(()));
        assert_eq!(setup.td_json_file_contents(), crate::INIT_TD_TENDRILS_JSON);
        assert!(api.is_tendrils_dir(&setup.td_dir))
    }
    else {
        assert_eq!(actual, Err(InitError::NotEmpty));
        assert!(!api.is_tendrils_dir(&setup.td_dir))
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_is_already_td_dir_returns_already_init_error_even_if_invalid_json(
    #[case] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_dot_td_dir();
    let json_content = "Invalid json content";
    write(&setup.td_json_file, json_content).unwrap();
    assert!(api.is_tendrils_dir(&setup.td_dir));
    assert!(parse_config(json_content).is_err());

    let actual = api.init_tendrils_dir(&setup.td_dir, force);

    assert_eq!(setup.td_json_file_contents(), json_content);
    assert_eq!(actual, Err(InitError::AlreadyInitialized));
}
