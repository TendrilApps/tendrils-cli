use crate::{link_tendril, symlink};
use crate::errors::TendrilActionError;
use crate::resolved_tendril::{ResolvedTendril, TendrilMode};
use crate::test_utils::{get_disposable_folder, is_empty};
use rstest::rstest;
use std::path::PathBuf;
use std::fs::{create_dir_all, read_to_string, write};
use tempdir::TempDir;

#[rstest]
fn given_tendril_is_not_link_mode_returns_mode_mismatch_error(
    #[values(TendrilMode::FolderMerge, TendrilMode::FolderOverwrite)]
    mode: TendrilMode,
    #[values(true, false)]
    dry_run: bool,
) {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let tendrils_folder = temp_parent_folder.path().join("TendrilsFolder");

    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_folder.path().to_path_buf(),
        mode,
    ).unwrap();

    let actual = link_tendril(
        &tendrils_folder,
        &given_tendril,
        dry_run
    );

    assert!(matches!(actual, Err(TendrilActionError::ModeMismatch)));
}

#[rstest]
#[case(true)]
#[case(false)]
fn dest_parent_doesnt_exist_returns_io_error_not_found(#[case] dry_run: bool) {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let tendrils_folder = temp_parent_folder.path().join("TendrilsFolder");
    let target = tendrils_folder.join("SomeApp").join("misc.txt");
    create_dir_all(tendrils_folder.join("SomeApp")).unwrap();
    write(&target, "Target file contents").unwrap();

    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        PathBuf::from("SomePathThatDoesNotExist"),
        TendrilMode::Link,
    ).unwrap();

    let actual = link_tendril(
        &tendrils_folder,
        &given_tendril,
        dry_run
    ).unwrap_err();

    match actual {
        TendrilActionError::IoError(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        _ => panic!("Actual error: {:?}", actual),
    }
    let target_file_contents = read_to_string(target).unwrap();
    assert_eq!(target_file_contents, "Target file contents");
    assert_eq!(temp_parent_folder.path().read_dir().iter().count(), 1);
}

#[rstest]
#[case(true)]
#[case(false)]
fn target_doesnt_exist_returns_io_error_not_found(#[case] dry_run: bool) {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let tendrils_folder = temp_parent_folder.path().join("TendrilsFolder");
    let dest = temp_parent_folder.path().join("misc.txt");

    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_folder.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();

    let actual = link_tendril(
        &tendrils_folder,
        &given_tendril,
        dry_run
    ).unwrap_err();

    match actual {
        TendrilActionError::IoError(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        _ => panic!("Actual error: {:?}", actual),
    }
    assert!(!dest.exists());
    assert!(is_empty(temp_parent_folder.path()));
}

#[rstest]
#[case(true)]
#[case(false)]
fn dest_exists_and_is_not_symlink_returns_type_mismatch_error(#[case] dry_run: bool) {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let tendrils_folder = temp_parent_folder.path().join("TendrilsFolder");
    let dest_file = temp_parent_folder.path().join("misc.txt");
    let dest_folder = temp_parent_folder.path().join("misc");
    let dest_nested = dest_folder.join("nested.txt");
    write(&dest_file, "Dest file contents").unwrap();
    create_dir_all(&dest_folder).unwrap();
    write(&dest_nested, "Nested file contents").unwrap();

    let given_file_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_folder.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();
    let given_folder_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc".to_string(),
        temp_parent_folder.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();

    let file_actual = link_tendril(
        &tendrils_folder,
        &given_file_tendril,
        dry_run
    ).unwrap_err();
    let folder_actual = link_tendril(
        &tendrils_folder,
        &given_folder_tendril,
        dry_run
    ).unwrap_err();

    let dest_file_contents = read_to_string(dest_file).unwrap();
    let dest_nested_contents = read_to_string(dest_nested).unwrap();
    assert!(matches!(file_actual, TendrilActionError::TypeMismatch));
    assert!(matches!(folder_actual, TendrilActionError::TypeMismatch));
    assert_eq!(dest_file_contents, "Dest file contents");
    assert_eq!(dest_nested_contents, "Nested file contents");
    assert!(is_empty(&tendrils_folder));
}

#[rstest]
#[case(true)]
#[case(false)]
fn target_is_symlink_returns_type_mismatch_error(#[case] dry_run: bool) {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let tendrils_folder = temp_parent_folder.path().join("TendrilsFolder");
    let dest = temp_parent_folder.path().join("misc.txt");
    let target = tendrils_folder.join("SomeApp").join("misc.txt");
    let nested_target = temp_parent_folder.path().join("original.txt");
    create_dir_all(target.parent().unwrap()).unwrap();
    write(&nested_target, "Orig file contents").unwrap();
    symlink(&target, &nested_target, false).unwrap();
    // TODO: Test with symdir

    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_folder.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();

    let file_actual = link_tendril(
        &tendrils_folder,
        &given_tendril,
        dry_run
    ).unwrap_err();

    let nested_target_file_contents = read_to_string(nested_target).unwrap();
    assert_eq!(nested_target_file_contents, "Orig file contents");
    assert!(matches!(file_actual, TendrilActionError::TypeMismatch));
    assert!(!dest.exists());
    assert!(target.is_symlink());
    assert_eq!(tendrils_folder.read_dir().iter().count(), 1);
}

#[test]
fn dest_doesnt_exist_but_parent_does_symlink_is_created() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let tendrils_folder = temp_parent_folder.path().join("TendrilsFolder");
    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_folder.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();
    let orig_target_file = temp_parent_folder.path().join("SomeRandomPath").join("some_orig_target.txt");
    let new_target_file = tendrils_folder.join("SomeApp").join("misc.txt");
    create_dir_all(orig_target_file.parent().unwrap()).unwrap();
    create_dir_all(new_target_file.parent().unwrap()).unwrap();
    write(&orig_target_file, "Orig target file contents").unwrap();
    write(&new_target_file, "New target file contents").unwrap();
    let dest = temp_parent_folder.path().join("misc.txt");
    assert!(!dest.exists());
    // TODO: Test with symdir

    link_tendril(
        &tendrils_folder,
        &given_tendril,
        false,
    ).unwrap();

    let dest_file_contents = read_to_string(&dest).unwrap();
    let orig_target_file_contents = read_to_string(orig_target_file).unwrap();
    assert!(dest.is_symlink());
    assert_eq!(dest_file_contents, "New target file contents");
    assert_eq!(orig_target_file_contents, "Orig target file contents");
}

#[test]
fn dest_doesnt_exist_but_parent_does_symlink_not_created_in_dry_run() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let tendrils_folder = temp_parent_folder.path().join("TendrilsFolder");
    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_folder.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();
    let orig_target_file = temp_parent_folder.path().join("SomeRandomPath").join("some_orig_target.txt");
    let new_target_file = tendrils_folder.join("SomeApp").join("misc.txt");
    create_dir_all(orig_target_file.parent().unwrap()).unwrap();
    create_dir_all(new_target_file.parent().unwrap()).unwrap();
    write(&orig_target_file, "Orig target file contents").unwrap();
    write(&new_target_file, "New target file contents").unwrap();
    let dest = temp_parent_folder.path().join("misc.txt");
    assert!(!dest.exists());
    // TODO: Test with symdir

    let actual = link_tendril(
        &tendrils_folder,
        &given_tendril,
        true,
    );

    assert!(matches!(actual, Err(TendrilActionError::Skipped)));
    assert!(!dest.exists());
    let orig_target_file_contents = read_to_string(orig_target_file).unwrap();
    let new_target_file_contents = read_to_string(new_target_file).unwrap();
    assert_eq!(orig_target_file_contents, "Orig target file contents");
    assert_eq!(new_target_file_contents, "New target file contents");
}


#[test]
fn existing_symlinks_at_dest_are_overwritten() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let tendrils_folder = temp_parent_folder.path().join("TendrilsFolder");
    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_folder.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();
    let orig_target_file = temp_parent_folder.path().join("SomeRandomPath").join("some_orig_target.txt");
    let new_target_file = tendrils_folder.join("SomeApp").join("misc.txt");
    create_dir_all(orig_target_file.parent().unwrap()).unwrap();
    create_dir_all(new_target_file.parent().unwrap()).unwrap();
    write(&orig_target_file, "Orig target file contents").unwrap();
    write(&new_target_file, "New target file contents").unwrap();
    let dest = temp_parent_folder.path().join("misc.txt");
    symlink(&dest, &orig_target_file, false).unwrap();
    // TODO: Test with symdir

    link_tendril(
        &tendrils_folder,
        &given_tendril,
        false,
    ).unwrap();

    let dest_file_contents = read_to_string(&dest).unwrap();
    let orig_target_file_contents = read_to_string(orig_target_file).unwrap();
    assert!(dest.is_symlink());
    assert_eq!(dest_file_contents, "New target file contents");
    assert_eq!(orig_target_file_contents, "Orig target file contents");
}

#[test]
fn existing_symlinks_at_dest_are_unmodified_in_dry_run() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let tendrils_folder = temp_parent_folder.path().join("TendrilsFolder");
    let given_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        temp_parent_folder.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();
    let orig_target_file = temp_parent_folder.path().join("SomeRandomPath").join("some_orig_target.txt");
    let new_target_file = tendrils_folder.join("SomeApp").join("misc.txt");
    create_dir_all(orig_target_file.parent().unwrap()).unwrap();
    create_dir_all(new_target_file.parent().unwrap()).unwrap();
    write(&orig_target_file, "Orig target file contents").unwrap();
    write(&new_target_file, "New target file contents").unwrap();
    let dest = temp_parent_folder.path().join("misc.txt");
    symlink(&dest, &orig_target_file, false).unwrap();
    // TODO: Test with symdir

    let actual = link_tendril(
        &tendrils_folder,
        &given_tendril,
        true,
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
