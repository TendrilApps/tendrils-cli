use crate::test_utils::{dot_td_dir, get_disposable_dir, repo_path_file, set_ra};
use crate::{TendrilsActor, TendrilsApi};
use rstest::rstest;
use serial_test::serial;
use std::env::set_var;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;
use tempdir::TempDir;

#[test]
#[serial("mut-env-var-testing")]
fn repo_path_file_does_not_exist_returns_none() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    set_var("HOME", temp.path());
    assert!(!repo_path_file().exists());

    let actual = api.get_default_repo_path();

    assert_eq!(actual.unwrap(), None);
}

#[test]
#[serial("mut-env-var-testing")]
fn repo_path_file_is_empty_returns_none() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    set_var("HOME", temp.path());
    let repo_path_file = repo_path_file();
    create_dir_all(dot_td_dir()).unwrap();
    write(&repo_path_file, "").unwrap();

    let actual = api.get_default_repo_path();

    assert_eq!(actual.unwrap(), None);
}

#[test]
#[serial("mut-env-var-testing")]
fn repo_path_file_no_read_access_returns_io_permission_error() {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    set_var("HOME", temp.path());
    let repo_path_file = repo_path_file();
    create_dir_all(dot_td_dir()).unwrap();
    write(&repo_path_file, "").unwrap();
    set_ra(&repo_path_file, false);

    let actual = api.get_default_repo_path();

    set_ra(&repo_path_file, true);
    assert_eq!(
        actual.unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied,
    );
}

#[rstest]
#[case("Some/Path")]
#[case("Some\\Path")]
#[case("I Do Not Exist")]
#[case("Multi\nLine\nString")]
#[serial("mut-env-var-testing")]
fn repo_path_file_exists_returns_path_even_if_invalid(#[case] file_contents: &str) {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    set_var("HOME", temp.path());
    let repo_path_file = repo_path_file();
    create_dir_all(dot_td_dir()).unwrap();
    write(&repo_path_file, &file_contents).unwrap();

    let actual = api.get_default_repo_path();

    assert_eq!(actual.unwrap(), Some(PathBuf::from(file_contents)));
}

#[rstest]
#[case(" SomePath ", " SomePath")]
#[case("\tSomePath\t", "\tSomePath")]
#[case("\rSomePath\r", "\rSomePath")]
#[case("SomePath\n", "SomePath")]
#[case("\nSomePath\n ", "\nSomePath")]
#[case("\nSome\nPath\n ", "\nSome\nPath")]
#[case("\r\n \tSomePath\r\n \t", "\r\n \tSomePath")]
#[serial("mut-env-var-testing")]
fn trailing_whitespace_is_removed_leading_whitespace_remains(
    #[case] file_contents: &str,
    #[case] exp_path_str: &str,
) {
    let api = TendrilsActor {};
    let temp = TempDir::new_in(get_disposable_dir(), "Temp").unwrap();
    set_var("HOME", temp.path());
    let repo_path_file = repo_path_file();
    create_dir_all(dot_td_dir()).unwrap();
    write(&repo_path_file, &file_contents).unwrap();

    let actual = api.get_default_repo_path();

    assert_eq!(actual.unwrap(), Some(PathBuf::from(exp_path_str)));
}
