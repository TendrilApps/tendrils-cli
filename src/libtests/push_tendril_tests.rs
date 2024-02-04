use crate::{push_tendril, ResolvedTendril, TendrilMode};
use crate::test_utils::get_disposable_dir;
use crate::errors::TendrilActionError;
use rstest::rstest;
use std::fs::{create_dir_all, read_to_string, write};
use tempdir::TempDir;

#[rstest]
fn given_link_mode_tendril_returns_mode_mismatch_error(
    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let td_dir = temp_parent_dir.path().join("TendrilsDir");
    let source = td_dir.join("SomeApp").join("misc.txt");
    let dest = temp_parent_dir.path().join("misc.txt");
    create_dir_all(source.parent().unwrap()).unwrap();
    write(&source, "Source file contents").unwrap();
    write(&dest, "Dest file contents").unwrap();
    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();

    let actual = push_tendril(&td_dir, &given_tendril, dry_run, force);

    let source_file_contents = read_to_string(&source).unwrap();
    let dest_file_contents = read_to_string(&dest).unwrap();
    assert!(matches!(actual, Err(TendrilActionError::ModeMismatch)));
    assert_eq!(source_file_contents, "Source file contents");
    assert_eq!(dest_file_contents, "Dest file contents");
}
