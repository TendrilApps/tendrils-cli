use crate::get_tendrils_folder;
use crate::utests::common::get_disposable_folder;
use serial_test::serial;
use tempdir::TempDir;

#[test]
#[serial]
fn starting_dir_not_tendrils_folder_returns_none() {
    let temp = TempDir::new_in(get_disposable_folder(), "Empty").unwrap();

    let actual = get_tendrils_folder(&temp.path());

    assert!(actual.is_none());
}

#[test]
#[serial]
fn starting_dir_is_tendrils_folder_returns_starting_dir() {
    let temp = TempDir::new_in(
        get_disposable_folder(),
        "EmptyTendrilsJson"
    ).unwrap();
    std::fs::File::create(temp.path().join("tendrils.json")).unwrap();

    let actual = get_tendrils_folder(&temp.path()).unwrap();

    assert_eq!(actual, temp.path());
}
