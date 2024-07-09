use crate::config::{Config, parse_config};
use crate::test_utils::get_disposable_dir;
use crate::{
    get_config,
    init_tendrils_dir,
    is_tendrils_dir,
    InitError,
    TendrilBundle,
};
use rstest::rstest;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;
use tempdir::TempDir;

#[rstest]
#[case(true)]
#[case(false)]
fn creates_valid_tendrils_json_file_in_empty_dir(#[case] force: bool) {
    let temp_init_dir =
        TempDir::new_in(get_disposable_dir(), "InitFolder").unwrap();
    let td_json_file = temp_init_dir.path().join("tendrils.json");
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

    let actual = init_tendrils_dir(temp_init_dir.path(), force);

    let td_json_contents = read_to_string(td_json_file).unwrap();
    assert_eq!(actual, Ok(()));
    assert_eq!(td_json_contents, crate::INIT_TD_TENDRILS_JSON);
    assert_eq!(get_config(temp_init_dir.path()).unwrap(), expected);
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_doesnt_exist_returns_io_error_not_found(#[case] force: bool) {
    let dir = PathBuf::from("I do not exist");

    let actual = init_tendrils_dir(&dir, force);

    assert!(!dir.join("tendrils.json").exists());
    assert_eq!(
        actual,
        Err(InitError::IoError { kind: std::io::ErrorKind::NotFound })
    );
}

#[rstest]
#[rstest]
#[case(true)]
#[case(false)]
fn dir_contains_another_misc_file_returns_not_empty_error_unless_forced(
    #[case] force: bool,
) {
    let temp_init_dir =
        TempDir::new_in(get_disposable_dir(), "InitFolder").unwrap();
    let td_json_file = temp_init_dir.path().join("tendrils.json");
    let misc_file = temp_init_dir.path().join("misc.txt");
    write(&misc_file, "Misc file contents").unwrap();

    let actual = init_tendrils_dir(&temp_init_dir.path(), force);

    assert_eq!(read_to_string(misc_file).unwrap(), "Misc file contents");
    if force {
        assert_eq!(actual, Ok(()));
        assert_eq!(
            read_to_string(td_json_file).unwrap(),
            crate::INIT_TD_TENDRILS_JSON
        );
    }
    else {
        assert_eq!(actual, Err(InitError::NotEmpty));
        assert!(!td_json_file.exists());
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_contains_another_misc_dir_returns_not_empty_error_unless_forced(
    #[case] force: bool,
) {
    let temp_init_dir =
        TempDir::new_in(get_disposable_dir(), "InitFolder").unwrap();
    let td_json_file = temp_init_dir.path().join("tendrils.json");
    let misc_dir = temp_init_dir.path().join("misc");
    let misc_nested = misc_dir.join("nested.txt");
    create_dir_all(&misc_dir).unwrap();
    write(&misc_nested, "Nested file contents").unwrap();

    let actual = init_tendrils_dir(&temp_init_dir.path(), force);

    assert_eq!(read_to_string(misc_nested).unwrap(), "Nested file contents");
    if force {
        assert_eq!(actual, Ok(()));
        assert_eq!(
            read_to_string(td_json_file).unwrap(),
            crate::INIT_TD_TENDRILS_JSON
        );
    }
    else {
        assert_eq!(actual, Err(InitError::NotEmpty));
        assert!(!td_json_file.exists());
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_contains_a_td_json_file_returns_already_init_error_even_if_invalid_json(
    #[case] force: bool,
) {
    let temp_init_dir =
        TempDir::new_in(get_disposable_dir(), "InitFolder").unwrap();
    let td_json_file = temp_init_dir.path().join("tendrils.json");
    let json_content = "Invalid json content";
    write(&td_json_file, json_content).unwrap();
    assert!(is_tendrils_dir(temp_init_dir.path()));
    assert!(parse_config(json_content).is_err());

    let actual = init_tendrils_dir(&temp_init_dir.path(), force);

    assert_eq!(read_to_string(td_json_file).unwrap(), json_content);
    assert_eq!(actual, Err(InitError::AlreadyInitialized));
}
