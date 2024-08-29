use crate::{get_tendrils_repo, is_tendrils_repo, GetTendrilsRepoError};
use crate::path_ext::{PathExt, UniPath};
use crate::test_utils::{
    default_repo_path_as_json,
    global_cfg_file,
    Setup,
};
use serial_test::serial;
use std::fs::{create_dir_all, write};
use std::path::{MAIN_SEPARATOR as SEP, PathBuf};

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_invalid_default_not_set_returns_given_invalid_err() {
    let setup = Setup::new();
    let starting_td_repo = setup.uni_td_repo();
    setup.set_home_dir();
    assert!(!global_cfg_file().exists());
    assert!(!is_tendrils_repo(&starting_td_repo));

    let actual = get_tendrils_repo(Some(&starting_td_repo));

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::GivenInvalid {
            path: starting_td_repo.inner().to_path_buf(),
        })
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_invalid_default_invalid_returns_given_invalid_err() {
    let setup = Setup::new();
    let starting_td_repo = setup.uni_td_repo();
    setup.make_global_cfg_file(default_repo_path_as_json("I DON'T EXIST"));
    assert!(!is_tendrils_repo(&starting_td_repo));

    let actual = get_tendrils_repo(Some(&starting_td_repo));

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::GivenInvalid {
            path: starting_td_repo.inner().to_path_buf(),
        })
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_invalid_default_valid_returns_given_invalid_err() {
    let setup = Setup::new();
    let starting_td_repo = UniPath::from(
        setup.temp_dir.path().join("I don't exist")
    );
    let default_td_repo = setup.uni_td_repo();
    setup.make_td_json_file(&[]);
    setup.make_global_cfg_file(
        default_repo_path_as_json(&default_td_repo.inner().to_string_lossy()),
    );
    assert!(!is_tendrils_repo(&starting_td_repo));
    assert!(is_tendrils_repo(&default_td_repo));

    let actual = get_tendrils_repo(Some(&starting_td_repo));

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::GivenInvalid {
            path: starting_td_repo.inner().to_path_buf(),
        })
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_valid_default_not_set_returns_starting_dir() {
    let setup = Setup::new();
    let starting_td_repo = setup.uni_td_repo();
    setup.make_td_json_file(&[]);
    setup.set_home_dir();
    assert!(!global_cfg_file().exists());
    assert!(is_tendrils_repo(&starting_td_repo));

    let actual = get_tendrils_repo(Some(&starting_td_repo)).unwrap();

    assert_eq!(actual, starting_td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_valid_default_valid_returns_starting_dir() {
    let setup = Setup::new();
    let starting_td_repo = setup.uni_td_repo();
    let default_td_repo = UniPath::from(
        setup.temp_dir.path().join("DefaultTendrilsRepo")
    );
    let default_dot_td_dir = default_td_repo.inner().join(".tendrils");
    setup.make_td_json_file(&[]);
    create_dir_all(&default_dot_td_dir).unwrap();
    write(default_dot_td_dir.join("tendrils.json"), "").unwrap();
    setup.make_global_cfg_file(
        default_repo_path_as_json(&default_td_repo.inner().to_string_lossy())
    );
    assert!(is_tendrils_repo(&starting_td_repo));
    assert!(is_tendrils_repo(&default_td_repo));

    let actual = get_tendrils_repo(Some(&starting_td_repo)).unwrap();

    assert_eq!(actual, starting_td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_valid_default_invalid_returns_starting_dir() {
    let setup = Setup::new();
    let starting_td_repo = setup.uni_td_repo();
    let default_td_repo = UniPath::from(
        setup.temp_dir.path().join("DefaultTendrilsRepo")
    );
    setup.make_td_json_file(&[]);
    setup.make_global_cfg_file(
        default_repo_path_as_json(&default_td_repo.inner().to_string_lossy())
    );
    assert!(is_tendrils_repo(&starting_td_repo));
    assert!(!is_tendrils_repo(&default_td_repo));

    let actual = get_tendrils_repo(Some(&starting_td_repo)).unwrap();

    assert_eq!(actual, starting_td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_none_default_not_set_returns_default_not_set_err() {
    let setup = Setup::new();
    let starting_td_repo = None;
    setup.set_home_dir();
    assert!(!global_cfg_file().exists());

    let actual = get_tendrils_repo(starting_td_repo);

    assert_eq!(actual, Err(GetTendrilsRepoError::DefaultNotSet));
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_none_default_invalid_returns_default_invalid_err() {
    let setup = Setup::new();
    let starting_td_repo = None;
    let default_td_repo = UniPath::from(PathBuf::from("I DON'T EXIST"));
    setup.make_global_cfg_file(
        default_repo_path_as_json("I DON'T EXIST")
    );
    assert!(!is_tendrils_repo(&default_td_repo));

    let actual = get_tendrils_repo(starting_td_repo);

    assert_eq!(
        actual,
        Err(GetTendrilsRepoError::DefaultInvalid {
            path: default_td_repo.inner().to_path_buf(),
        })
    );
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_none_default_valid_returns_default() {
    let setup = Setup::new();
    let starting_td_repo = None;
    let default_td_repo = UniPath::from(setup.td_repo.clone());
    let json_path =
        default_td_repo.inner().to_string_lossy().replace("\\", "\\\\");
    setup.make_td_json_file(&[]);
    setup.make_global_cfg_file(default_repo_path_as_json(&json_path));
    assert!(is_tendrils_repo(&default_td_repo));

    let actual = get_tendrils_repo(starting_td_repo).unwrap();

    assert_eq!(actual, default_td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn starting_dir_is_default_dir_and_is_valid_returns_dir() {
    let setup = Setup::new();
    let starting_td_repo = setup.uni_td_repo();
    let default_td_repo = setup.uni_td_repo();
    let json_path =
        default_td_repo.inner().to_string_lossy().replace("\\", "\\\\");
    setup.make_td_json_file(&[]);
    setup.make_global_cfg_file(default_repo_path_as_json(&json_path));
    assert!(is_tendrils_repo(&starting_td_repo));
    assert!(is_tendrils_repo(&default_td_repo));
    assert_eq!(&starting_td_repo, &default_td_repo);

    let actual = get_tendrils_repo(Some(&starting_td_repo)).unwrap();

    assert_eq!(actual, default_td_repo);
}

#[test]
#[serial("mut-env-var-testing")]
fn leading_tilde_and_env_vars_in_given_path_are_resolved_and_dir_seps_are_replaced() {
    let setup = Setup::new();
    setup.make_td_json_file(&[]);
    setup.set_home_dir();
    std::env::set_var("var", "TendrilsRepo");
    let starting_td_repo = UniPath::from(PathBuf::from("~/<var>\\"));
    let expected_str = format!(
        "{}{SEP}TendrilsRepo",
        setup.temp_dir.path().to_string_lossy(),
    );

    let actual = get_tendrils_repo(Some(&starting_td_repo)).unwrap();

    assert_eq!(actual.inner().to_string_lossy(), expected_str);
}

#[test]
#[serial("mut-env-var-testing")]
fn leading_tilde_and_env_vars_in_default_path_are_resolved_and_dir_seps_are_replaced() {
    let setup = Setup::new();
    setup.make_td_json_file(&[]);
    setup.make_global_cfg_file(
        default_repo_path_as_json("~/<var>\\\\"),
    );
    std::env::set_var("var", "TendrilsRepo");
    let expected_str = format!(
        "{}{SEP}TendrilsRepo",
        setup.temp_dir.path().to_string_lossy(),
    );

    let actual = get_tendrils_repo(None).unwrap();

    assert_eq!(actual.inner().to_string_lossy(), expected_str);
}

#[test]
#[serial("mut-env-var-testing")]
fn leading_tilde_and_env_vars_in_given_path_are_resolved_in_error_path_and_dir_seps_are_replaced() {
    let setup = Setup::new();
    setup.set_home_dir();
    std::env::set_var("var", "TendrilsRepo");
    let starting_td_repo = UniPath::from(PathBuf::from("~/<var>\\"));
    let expected_str = format!(
        "{}{SEP}TendrilsRepo",
        setup.temp_dir.path().to_string_lossy(),
    );

    let actual = get_tendrils_repo(Some(&starting_td_repo));

    if let Err(GetTendrilsRepoError::GivenInvalid { path: p }) = actual {
        assert_eq!(p.to_string_lossy(), expected_str);
    }
    else {
        panic!();
    }
}

#[test]
#[serial("mut-env-var-testing")]
fn leading_tilde_and_env_vars_in_default_path_are_resolved_in_error_path_and_dir_seps_are_replaced() {
    let setup = Setup::new();
    setup.make_global_cfg_file(
        default_repo_path_as_json("~/<var>\\\\"),
    );
    std::env::set_var("var", "TendrilsRepo");
    let expected_str = format!(
        "{}{SEP}TendrilsRepo",
        setup.temp_dir.path().to_string_lossy(),
    );

    let actual = get_tendrils_repo(None);

    if let Err(GetTendrilsRepoError::DefaultInvalid { path: p }) = actual {
        assert_eq!(p.to_string_lossy(), expected_str);
    }
    else {
        panic!();
    }
}

#[test]
fn given_path_is_absoluted_and_reduced() {
    let setup = Setup::new();
    setup.make_td_json_file(&[]);

    let starting_td_repo =
        setup.td_repo.join_raw(&PathBuf::from("\\/.////SkipMe\\\\.."));
    // See issues with path_clean crate and drive letters on Windows
    #[cfg(not(windows))]
    let starting_td_repo = PathBuf::from(".././").join_raw(&starting_td_repo);

    let expected_str = format!(
        "{}{SEP}TendrilsRepo",
        setup.temp_dir.path().to_string_lossy(),
    );

    let actual = get_tendrils_repo(Some(&starting_td_repo.into())).unwrap();

    assert_eq!(actual.inner().to_string_lossy(), expected_str);
}

#[test]
#[serial("mut-env-var-testing")]
fn relative_default_path_is_absoluted_and_reduced() {
    let setup = Setup::new();
    let default_td_repo = 
        setup.td_repo.join_raw(&PathBuf::from("\\/.////SkipMe\\\\.."));
    // See issues with path_clean crate and drive letters on Windows
    #[cfg(not(windows))]
    let default_td_repo = PathBuf::from(".././").join_raw(&default_td_repo);

    setup.make_td_json_file(&[]);
    setup.make_global_cfg_file( default_repo_path_as_json(
        &default_td_repo.to_string_lossy().replace("\\", "\\\\")
    ));
    let expected_str = format!(
        "{}{SEP}TendrilsRepo",
        setup.temp_dir.path().to_string_lossy(),
    );

    let actual = get_tendrils_repo(None).unwrap();

    assert_eq!(actual.inner().to_string_lossy(), expected_str);
}

#[test]
fn relative_given_path_is_absoluted_and_reduced_in_error_path() {
    let starting_td_repo =
        UniPath::from(PathBuf::from(".././Some\\\\Rel//..\\Path"));
    let expected_str = format!("{SEP}Some{SEP}Path");

    let actual = get_tendrils_repo(Some(&starting_td_repo));

    if let Err(GetTendrilsRepoError::GivenInvalid { path: p }) = actual {
        assert_eq!(p.to_string_lossy(), expected_str);
    }
    else {
        panic!("{:?}", actual);
    }
}

#[test]
#[serial("mut-env-var-testing")]
fn relative_default_path_is_absoluted_and_reduced_in_error_path() {
    let setup = Setup::new();
    setup.make_global_cfg_file(default_repo_path_as_json(
        &".././Some\\\\Rel//..\\Path".replace("\\", "\\\\")
    ));
    let expected_str = format!("{SEP}Some{SEP}Path");

    let actual = get_tendrils_repo(None);

    if let Err(GetTendrilsRepoError::DefaultInvalid { path: p }) = actual {
        assert_eq!(p.to_string_lossy(), expected_str);
    }
    else {
        panic!("{:?}", actual);
    }
}
