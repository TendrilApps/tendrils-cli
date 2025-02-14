use crate::config::{Config, parse_config_expose};
use crate::test_utils::{
    global_cfg_dir,
    global_cfg_file,
    home_dir,
    Setup
};
use crate::{
    is_tendrils_repo,
    InitError,
    RawTendril,
    TendrilMode,
    TendrilsActor,
    TendrilsApi,
    UniPath,
};
use rstest::rstest;
use serial_test::serial;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;

#[rstest]
#[case(true)]
#[case(false)]
fn creates_dot_tendrils_dir_and_contents_in_empty_dir(#[case] force: bool) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_td_repo_dir();
    let expected_t1 = RawTendril {
        local: "SomeApp/SomeFile.ext".to_string(),
        remote: "/path/to/SomeFile.ext".to_string(),
        mode: TendrilMode::DirOverwrite,
        profiles: vec![],
    };
    let expected_t2 = RawTendril {
        local: "SomeApp2/SomeFolder".to_string(),
        remote: "/path/to/SomeFolder".to_string(),
        mode: TendrilMode::Link,
        profiles: vec!["home".to_string(), "work".to_string()],
    };
    let expected_t3 = RawTendril {
        local: "SomeApp2/SomeFolder".to_string(),
        remote: "/path/to/DifferentName".to_string(),
        mode: TendrilMode::Link,
        profiles: vec!["home".to_string(), "work".to_string()],
    };
    let expected_t4 = RawTendril {
        local: "SomeApp2/SomeFolder".to_string(),
        remote: "~/path/in/home/dir/SomeFolder".to_string(),
        mode: TendrilMode::Link,
        profiles: vec!["home".to_string(), "work".to_string()],
    };
    let expected_t5 = RawTendril {
        local: "SomeApp2/SomeFolder".to_string(),
        remote: "/path/using/<MY-ENV-VAR>/SomeFolder".to_string(),
        mode: TendrilMode::Link,
        profiles: vec!["home".to_string(), "work".to_string()],
    };
    let expected_t6 = RawTendril {
        local: "SomeApp3/file.txt".to_string(),
        remote: "~/unix/specific/path/file.txt".to_string(),
        mode: TendrilMode::Link,
        profiles: vec!["unix".to_string()],
    };
    let expected_t7 = RawTendril {
        local: "SomeApp3/file.txt".to_string(),
        remote: "~/windows/specific/path/file.txt".to_string(),
        mode: TendrilMode::DirOverwrite,
        profiles: vec!["windows".to_string()],
    };
    let expected_t8 = RawTendril {
        local: "SomeApp3/file.txt".to_string(),
        remote: "~/windows/another-specific/path/file.txt".to_string(),
        mode: TendrilMode::DirOverwrite,
        profiles: vec!["windows".to_string()],
    };
    let expected_tendrils = vec![
        expected_t1,
        expected_t2,
        expected_t3,
        expected_t4,
        expected_t5,
        expected_t6,
        expected_t7,
        expected_t8,
    ];
    let expected = Config { raw_tendrils: expected_tendrils };

    let actual = api.init_tendrils_repo(&setup.uni_td_repo(), force);

    assert_eq!(actual, Ok(()));
    assert_eq!(setup.td_json_file_contents(), crate::INIT_TD_TENDRILS_JSON);
    assert!(is_tendrils_repo(&setup.uni_td_repo()));
    assert_eq!(
        crate::config::get_config(&setup.uni_td_repo()).unwrap(),
        expected,
    );
}

#[rstest]
#[case(true)]
#[case(false)]
#[serial(SERIAL_CD)]
fn does_not_change_cd(#[case] force: bool) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_td_repo_dir();
    let cd = setup.td_repo.clone();
    std::env::set_current_dir(&cd).unwrap();

    let actual = api.init_tendrils_repo(&UniPath::from(&cd), force);

    assert_eq!(std::env::current_dir().unwrap(), cd);
    std::env::set_current_dir(&setup.temp_dir.path().parent().unwrap()).unwrap();

    assert_eq!(actual, Ok(()));
    assert!(is_tendrils_repo(&cd.into()));
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_doesnt_exist_returns_io_error_not_found(#[case] force: bool) {
    let api = TendrilsActor {};
    let dir = PathBuf::from("I do not exist");

    let actual = api.init_tendrils_repo(&UniPath::from(&dir), force);

    assert!(!dir.join("tendrils.json").exists());
    assert_eq!(
        actual,
        Err(InitError::IoError { kind: std::io::ErrorKind::NotFound })
    );
    assert!(!is_tendrils_repo(&dir.into()))
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_is_a_file_returns_io_err(
    #[case] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    write(&setup.td_repo, "I'm not a folder!").unwrap();

    let actual = api.init_tendrils_repo(&setup.uni_td_repo(), force);

    let file_contents = read_to_string(setup.td_repo).unwrap();
    assert_eq!(file_contents, "I'm not a folder!");
    matches!(actual, Err(InitError::IoError { kind: _ }));
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
    setup.make_td_repo_dir();
    let misc_file = setup.td_repo.join("misc.txt");
    write(&misc_file, "Misc file contents").unwrap();

    let actual = api.init_tendrils_repo(&setup.uni_td_repo(), force);

    assert_eq!(read_to_string(misc_file).unwrap(), "Misc file contents");
    if force {
        assert_eq!(actual, Ok(()));
        assert_eq!(setup.td_json_file_contents(), crate::INIT_TD_TENDRILS_JSON);
        assert!(is_tendrils_repo(&setup.uni_td_repo()))
    }
    else {
        assert_eq!(actual, Err(InitError::NotEmpty));
        assert!(!is_tendrils_repo(&setup.uni_td_repo()))
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
    let misc_dir = setup.td_repo.join("misc");
    let misc_nested = misc_dir.join("nested.txt");
    create_dir_all(&misc_dir).unwrap();
    write(&misc_nested, "Nested file contents").unwrap();

    let actual = api.init_tendrils_repo(&setup.uni_td_repo(), force);

    assert_eq!(read_to_string(misc_nested).unwrap(), "Nested file contents");
    if force {
        assert_eq!(actual, Ok(()));
        assert_eq!(setup.td_json_file_contents(), crate::INIT_TD_TENDRILS_JSON);
        assert!(is_tendrils_repo(&setup.uni_td_repo()))
    }
    else {
        assert_eq!(actual, Err(InitError::NotEmpty));
        assert!(!is_tendrils_repo(&setup.uni_td_repo()))
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

    let actual = api.init_tendrils_repo(&setup.uni_td_repo(), force);

    if force {
        assert_eq!(actual, Ok(()));
        assert_eq!(setup.td_json_file_contents(), crate::INIT_TD_TENDRILS_JSON);
        assert!(is_tendrils_repo(&setup.uni_td_repo()))
    }
    else {
        assert_eq!(actual, Err(InitError::NotEmpty));
        assert!(!is_tendrils_repo(&setup.uni_td_repo()))
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

    let actual = api.init_tendrils_repo(&setup.uni_td_repo(), force);

    assert_eq!(read_to_string(misc_nested).unwrap(), "Nested file contents");
    if force {
        assert_eq!(actual, Ok(()));
        assert_eq!(setup.td_json_file_contents(), crate::INIT_TD_TENDRILS_JSON);
        assert!(is_tendrils_repo(&setup.uni_td_repo()))
    }
    else {
        assert_eq!(actual, Err(InitError::NotEmpty));
        assert!(!is_tendrils_repo(&setup.uni_td_repo()))
    }
}

#[rstest]
#[case(true)]
#[case(false)]
#[serial(SERIAL_MUT_ENV_VARS)]
fn init_in_home_dir_with_global_cfg_file_returns_non_empty_error_unless_forced(
    #[case] force: bool,
) {
    let api = TendrilsActor {};
    let mut setup = Setup::new();
    setup.make_global_cfg_file("Cfg file contents".to_string());
    setup.td_repo = home_dir();
    setup.dot_td_dir = setup.td_repo.join(".tendrils");
    setup.td_json_file = setup.dot_td_dir.join("tendrils.json");

    let actual = api.init_tendrils_repo(&setup.uni_td_repo(), force);

    assert_eq!(read_to_string(global_cfg_file()).unwrap(), "Cfg file contents");
    if force {
        assert_eq!(actual, Ok(()));
        assert_eq!(setup.td_json_file_contents(), crate::INIT_TD_TENDRILS_JSON);
        assert!(is_tendrils_repo(&setup.uni_td_repo()))
    }
    else {
        assert_eq!(actual, Err(InitError::NotEmpty));
        assert!(!is_tendrils_repo(&setup.uni_td_repo()))
    }
}

#[rstest]
#[case(true)]
#[case(false)]
#[serial(SERIAL_MUT_ENV_VARS)]
fn init_in_global_cfg_dir_with_global_cfg_file_returns_non_empty_error_unless_forced(
    #[case] force: bool,
) {
    let api = TendrilsActor {};
    let mut setup = Setup::new();
    setup.make_global_cfg_file("Cfg file contents".to_string());
    setup.td_repo = global_cfg_dir();
    setup.dot_td_dir = setup.td_repo.join(".tendrils");
    setup.td_json_file = setup.dot_td_dir.join("tendrils.json");

    let actual = api.init_tendrils_repo(&setup.uni_td_repo(), force);

    assert_eq!(read_to_string(global_cfg_file()).unwrap(), "Cfg file contents");
    if force {
        assert_eq!(actual, Ok(()));
        assert_eq!(setup.td_json_file_contents(), crate::INIT_TD_TENDRILS_JSON);
        assert!(is_tendrils_repo(&setup.uni_td_repo()))
    }
    else {
        assert_eq!(actual, Err(InitError::NotEmpty));
        assert!(!is_tendrils_repo(&setup.uni_td_repo()))
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_is_already_td_repo_returns_already_init_error_even_if_invalid_json(
    #[case] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_dot_td_dir();
    let json_content = "Invalid json content";
    write(&setup.td_json_file, json_content).unwrap();
    assert!(is_tendrils_repo(&setup.uni_td_repo()));
    assert!(parse_config_expose(json_content).is_err());

    let actual = api.init_tendrils_repo(&setup.uni_td_repo(), force);

    assert_eq!(setup.td_json_file_contents(), json_content);
    assert_eq!(actual, Err(InitError::AlreadyInitialized));
}
