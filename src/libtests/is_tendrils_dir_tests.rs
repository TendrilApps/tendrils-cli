use crate::is_tendrils_dir;
use crate::test_utils::get_disposable_dir;
use tempdir::TempDir;

#[test]
fn empty_dir_returns_false() {
    let temp = TempDir::new_in(get_disposable_dir(), "Empty").unwrap();

    assert!(!is_tendrils_dir(&temp.path()));
}

#[test]
fn misc_other_files_only_returns_false() {
    let temp = TempDir::new_in(
        get_disposable_dir(),
        "MiscOtherFiles"
    ).unwrap();
    std::fs::File::create(temp.path().join("misc.txt")).unwrap();

    assert!(!is_tendrils_dir(&temp.path()));
}

#[test]
fn has_tendrils_json_dir_returns_false() {
    let temp = TempDir::new_in(
        get_disposable_dir(),
        "TendrilsJsonSubdir"
    ).unwrap();
    std::fs::create_dir(temp.path().join("tendrils.json")).unwrap();

    assert!(!is_tendrils_dir(&temp.path()));
}

#[test]
fn has_tendrils_json_file_returns_true() {
    let temp = TempDir::new_in(
        get_disposable_dir(),
        "EmptyTendrilsJson"
    ).unwrap();
    std::fs::File::create(temp.path().join("tendrils.json")).unwrap();

    assert!(is_tendrils_dir(&temp.path()));
}
