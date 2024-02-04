use crate::{link_tendril, symlink};
use crate::errors::TendrilActionError;
use crate::resolved_tendril::{ResolvedTendril, TendrilMode};
use crate::test_utils::{get_disposable_dir, is_empty};
use rstest::rstest;
use std::path::PathBuf;
use std::fs::{create_dir_all, read_to_string, write};
use tempdir::TempDir;

#[rstest]
fn given_tendril_is_not_link_mode_returns_mode_mismatch_error(
    #[values(TendrilMode::DirMerge, TendrilMode::DirOverwrite)]
    mode: TendrilMode,

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

    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_dir.path().to_path_buf(),
        mode,
    ).unwrap();

    let actual = link_tendril(
        &td_dir,
        &given_tendril,
        dry_run,
        force,
    );

    assert!(matches!(actual, Err(TendrilActionError::ModeMismatch)));
}

#[rstest]
#[case(true)]
#[case(false)]
fn dest_parent_doesnt_exist_returns_io_error_not_found(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let td_dir = temp_parent_dir.path().join("TendrilsDir");
    let target = td_dir.join("SomeApp").join("misc.txt");
    create_dir_all(td_dir.join("SomeApp")).unwrap();
    write(&target, "Target file contents").unwrap();

    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        PathBuf::from("SomePathThatDoesNotExist"),
        TendrilMode::Link,
    ).unwrap();

    let actual = link_tendril(
        &td_dir,
        &given_tendril,
        dry_run,
        force,
    ).unwrap_err();

    match actual {
        TendrilActionError::IoError(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        _ => panic!("Actual error: {:?}", actual),
    }
    let target_file_contents = read_to_string(target).unwrap();
    assert_eq!(target_file_contents, "Target file contents");
    assert_eq!(temp_parent_dir.path().read_dir().iter().count(), 1);
}

#[rstest]
#[case(true)]
#[case(false)]
fn target_doesnt_exist_returns_io_error_not_found(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let td_dir = temp_parent_dir.path().join("TendrilsDir");
    let dest = temp_parent_dir.path().join("misc.txt");

    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();

    let actual = link_tendril(
        &td_dir,
        &given_tendril,
        dry_run,
        force,
    ).unwrap_err();

    match actual {
        TendrilActionError::IoError(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        _ => panic!("Actual error: {:?}", actual),
    }
    assert!(!dest.exists());
    assert!(is_empty(temp_parent_dir.path()));
}

#[rstest]
#[case(true)]
#[case(false)]
fn dest_exists_and_is_not_symlink_returns_type_mismatch_error_unless_forced(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let td_dir = temp_parent_dir.path().join("TendrilsDir");
    let dest_file = temp_parent_dir.path().join("misc.txt");
    let dest_dir = temp_parent_dir.path().join("misc");
    let dest_nested_file = dest_dir.join("nested.txt");
    let target_file = td_dir.join("SomeApp").join("misc.txt");
    let target_dir = td_dir.join("SomeApp").join("misc");
    let target_nested = target_dir.join("nested.txt");
    write(&dest_file, "Dest file contents").unwrap();
    create_dir_all(&dest_dir).unwrap();
    create_dir_all(&target_dir).unwrap();
    write(&target_file, "Target file contents").unwrap();
    write(&dest_nested_file, "Dest nested file contents").unwrap();
    write(&target_nested, "Target nested file contents").unwrap();

    let given_file_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();
    let given_dir_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc".to_string(),
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();

    let file_actual = link_tendril(
        &td_dir,
        &given_file_tendril,
        dry_run,
        force,
    );
    let dir_actual = link_tendril(
        &td_dir,
        &given_dir_tendril,
        dry_run,
        force,
    );

    match (dry_run, force) {
        (_, false) => {
            assert!(matches!(file_actual, Err(TendrilActionError::TypeMismatch)));
            assert!(matches!(dir_actual, Err(TendrilActionError::TypeMismatch)));
        },
        (false, true) => {
            assert!(matches!(file_actual, Ok(())));
            assert!(matches!(dir_actual, Ok(())));
        },
        (true, true) => {
            assert!(matches!(file_actual, Err(TendrilActionError::Skipped)));
            assert!(matches!(dir_actual, Err(TendrilActionError::Skipped)));
        },
    }

    let dest_file_contents = read_to_string(&dest_file).unwrap();
    let dest_nested_contents = read_to_string(&dest_nested_file).unwrap();
    if force && !dry_run {
        assert!(dest_file.is_symlink());
        assert!(dest_dir.is_symlink());
        assert_eq!(dest_file_contents, "Target file contents");
        assert_eq!(dest_nested_contents, "Target nested file contents");
    }
    else {
        assert!(!dest_file.is_symlink());
        assert!(!dest_dir.is_symlink());
        assert_eq!(dest_file_contents, "Dest file contents");
        assert_eq!(dest_nested_contents, "Dest nested file contents");
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn target_is_symlink_returns_type_mismatch_error_unless_forced(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let td_dir = temp_parent_dir.path().join("TendrilsDir");
    let dest = temp_parent_dir.path().join("misc.txt");
    let target = td_dir.join("SomeApp").join("misc.txt");
    let final_target = temp_parent_dir.path().join("original.txt");
    create_dir_all(target.parent().unwrap()).unwrap();
    write(&final_target, "Orig file contents").unwrap();
    symlink(&target, &final_target, false, false).unwrap();
    // TODO: Test with symdir

    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();

    let file_actual = link_tendril(
        &td_dir,
        &given_tendril,
        dry_run,
        force,
    );

    match (dry_run, force) {
        (_, false) => {
            assert!(matches!(file_actual, Err(TendrilActionError::TypeMismatch)));
        },
        (false, true) => {
            assert!(matches!(file_actual, Ok(())));
        },
        (true, true) => {
            assert!(matches!(file_actual, Err(TendrilActionError::Skipped)));
        },
    }

    let nested_target_file_contents = read_to_string(final_target).unwrap();
    assert_eq!(nested_target_file_contents, "Orig file contents");
    assert!(target.is_symlink());
    assert_eq!(td_dir.read_dir().iter().count(), 1);
    if force && !dry_run {
        let dest_file_contents = read_to_string(dest).unwrap();
        assert_eq!(dest_file_contents, "Orig file contents");
    }
    else {
        assert!(!dest.exists());
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn dest_doesnt_exist_but_parent_does_symlink_is_created(#[case] force: bool) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let td_dir = temp_parent_dir.path().join("TendrilsDir");
    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();
    let orig_target_file = temp_parent_dir.path().join("SomeRandomPath").join("some_orig_target.txt");
    let new_target_file = td_dir.join("SomeApp").join("misc.txt");
    create_dir_all(orig_target_file.parent().unwrap()).unwrap();
    create_dir_all(new_target_file.parent().unwrap()).unwrap();
    write(&orig_target_file, "Orig target file contents").unwrap();
    write(&new_target_file, "New target file contents").unwrap();
    let dest = temp_parent_dir.path().join("misc.txt");
    assert!(!dest.exists());
    // TODO: Test with symdir

    link_tendril(
        &td_dir,
        &given_tendril,
        false,
        force,
    ).unwrap();

    let dest_file_contents = read_to_string(&dest).unwrap();
    let orig_target_file_contents = read_to_string(orig_target_file).unwrap();
    assert!(dest.is_symlink());
    assert_eq!(dest_file_contents, "New target file contents");
    assert_eq!(orig_target_file_contents, "Orig target file contents");
}

#[rstest]
#[case(true)]
#[case(false)]
fn dest_doesnt_exist_but_parent_does_symlink_not_created_in_dry_run(
    #[case] force: bool
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let td_dir = temp_parent_dir.path().join("TendrilsDir");
    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();
    let orig_target_file = temp_parent_dir.path().join("SomeRandomPath").join("some_orig_target.txt");
    let new_target_file = td_dir.join("SomeApp").join("misc.txt");
    create_dir_all(orig_target_file.parent().unwrap()).unwrap();
    create_dir_all(new_target_file.parent().unwrap()).unwrap();
    write(&orig_target_file, "Orig target file contents").unwrap();
    write(&new_target_file, "New target file contents").unwrap();
    let dest = temp_parent_dir.path().join("misc.txt");
    assert!(!dest.exists());
    // TODO: Test with symdir

    let actual = link_tendril(
        &td_dir,
        &given_tendril,
        true,
        force,
    );

    assert!(matches!(actual, Err(TendrilActionError::Skipped)));
    assert!(!dest.exists());
    let orig_target_file_contents = read_to_string(orig_target_file).unwrap();
    let new_target_file_contents = read_to_string(new_target_file).unwrap();
    assert_eq!(orig_target_file_contents, "Orig target file contents");
    assert_eq!(new_target_file_contents, "New target file contents");
}

#[rstest]
#[case(true)]
#[case(false)]
fn existing_symlinks_at_dest_are_overwritten(#[case] force: bool) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let td_dir = temp_parent_dir.path().join("TendrilsDir");
    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();
    let orig_target_file = temp_parent_dir.path().join("SomeRandomPath").join("some_orig_target.txt");
    let new_target_file = td_dir.join("SomeApp").join("misc.txt");
    create_dir_all(orig_target_file.parent().unwrap()).unwrap();
    create_dir_all(new_target_file.parent().unwrap()).unwrap();
    write(&orig_target_file, "Orig target file contents").unwrap();
    write(&new_target_file, "New target file contents").unwrap();
    let dest = temp_parent_dir.path().join("misc.txt");
    symlink(&dest, &orig_target_file, false, false).unwrap();
    // TODO: Test with symdir

    link_tendril(
        &td_dir,
        &given_tendril,
        false,
        force,
    ).unwrap();

    let dest_file_contents = read_to_string(&dest).unwrap();
    let orig_target_file_contents = read_to_string(orig_target_file).unwrap();
    assert!(dest.is_symlink());
    assert_eq!(dest_file_contents, "New target file contents");
    assert_eq!(orig_target_file_contents, "Orig target file contents");
}

#[rstest]
#[case(true)]
#[case(false)]
fn existing_symlinks_at_dest_are_unmodified_in_dry_run(#[case] force: bool) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let td_dir = temp_parent_dir.path().join("TendrilsDir");
    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();
    let orig_target_file = temp_parent_dir.path().join("SomeRandomPath").join("some_orig_target.txt");
    let new_target_file = td_dir.join("SomeApp").join("misc.txt");
    create_dir_all(orig_target_file.parent().unwrap()).unwrap();
    create_dir_all(new_target_file.parent().unwrap()).unwrap();
    write(&orig_target_file, "Orig target file contents").unwrap();
    write(&new_target_file, "New target file contents").unwrap();
    let dest = temp_parent_dir.path().join("misc.txt");
    symlink(&dest, &orig_target_file, false, false).unwrap();
    // TODO: Test with symdir

    let actual = link_tendril(
        &td_dir,
        &given_tendril,
        true,
        force,
    );

    assert!(matches!(actual, Err(TendrilActionError::Skipped)));
    let dest_file_contents = read_to_string(&dest).unwrap();
    let orig_target_file_contents = read_to_string(orig_target_file).unwrap();
    let new_target_file_contents = read_to_string(new_target_file).unwrap();
    assert!(dest.is_symlink());
    assert_eq!(dest_file_contents, "Orig target file contents");
    assert_eq!(orig_target_file_contents, "Orig target file contents");
    assert_eq!(new_target_file_contents, "New target file contents");
}
