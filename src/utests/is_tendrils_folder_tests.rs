use crate::is_tendrils_folder;
use crate::utests::common::get_disposable_folder;
use serial_test::serial;
use tempdir::TempDir;

#[test]
#[serial] // To avoid flaky tests upon first run (before the disposable folder exists)
fn empty_dir_returns_false() {
    let temp = TempDir::new_in(get_disposable_folder(), "Empty").unwrap();

    assert!(!is_tendrils_folder(&temp.path()));
}

#[test]
#[serial]
fn misc_other_files_only_returns_false() {
    let temp = TempDir::new_in(
        get_disposable_folder(),
        "MiscOtherFiles"
    ).unwrap();
    std::fs::File::create(temp.path().join("misc.txt")).unwrap();

    assert!(!is_tendrils_folder(&temp.path()));
}

#[test]
#[serial]
fn has_tendrils_json_dir_returns_false() {
    let temp = TempDir::new_in(
        get_disposable_folder(),
        "TendrilsJsonSubdir"
    ).unwrap();
    std::fs::create_dir(temp.path().join("tendrils.json")).unwrap();

    assert!(!is_tendrils_folder(&temp.path()));
}

#[test]
#[serial]
fn has_tendrils_json_file_returns_true() {
    let temp = TempDir::new_in(
        get_disposable_folder(),
        "EmptyTendrilsJson"
    ).unwrap();
    std::fs::File::create(temp.path().join("tendrils.json")).unwrap();

    assert!(is_tendrils_folder(&temp.path()));
}
