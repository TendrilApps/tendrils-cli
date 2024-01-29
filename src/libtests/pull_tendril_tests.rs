use crate::{
    is_tendrils_folder,
    pull_tendril,
    TendrilActionError,
};
use crate::resolved_tendril::{ResolvedTendril, TendrilMode};
use crate::test_utils::{
    get_disposable_folder,
    get_samples_folder,
    get_username_can_panic,
    is_empty,
    Setup,
    SetupOpts,
};
use rstest::rstest;
use std::fs::{
    create_dir_all,
    metadata,
    read_to_string,
    set_permissions,
    write,
};
use std::path::PathBuf;
use tempdir::TempDir;

#[rstest]
#[case("NoDot")]
#[case("single.dot")]
#[case("multi.sandwiched.dots")]
#[case(".LeadingDot")]
#[case("TrailingDot.")]
fn tendril_exists_at_source_path_copies_successfully(#[case] name: String) {
    let mut opts = SetupOpts::default();
    opts.source_filename = &name;
    opts.source_foldername = &name;
    opts.make_source_folder = false;
    let file_setup = Setup::new(&opts);
    opts.is_folder_tendril = true;
    opts.make_source_file = false;
    opts.make_source_folder = true;
    let folder_setup = Setup::new(&opts);

    pull_tendril(
        &file_setup.tendrils_dir,
        &file_setup.tendril,
        false
    ).unwrap();
    pull_tendril(
        &folder_setup.tendrils_dir,
        &folder_setup.tendril,
        false
    ).unwrap();

    assert_eq!(file_setup.dest_file_contents(), "Source file contents");
    assert_eq!(file_setup.tendrils_dir.join("SomeApp").read_dir().iter().count(), 1);
    assert_eq!(folder_setup.dest_nested_file_contents(), "Nested file contents");
    assert_eq!(folder_setup.tendrils_dir.join("SomeApp").read_dir().iter().count(), 1);
}

#[test]
fn tendril_exists_at_source_path_in_dry_run_returns_skipped_error_does_not_modify_dest() {
    // TODO: Test for symlink setup
    let mut opts = SetupOpts::default();
    opts.make_source_folder = false;
    let file_setup = Setup::new(&opts);
    opts.is_folder_tendril = true;
    opts.make_source_file = false;
    opts.make_source_folder = true;
    let folder_setup = Setup::new(&opts);
    create_dir_all(&file_setup.dest_file.parent().unwrap()).unwrap();
    create_dir_all(&folder_setup.dest_folder).unwrap();
    write(&file_setup.dest_file, "Dest file contents").unwrap();
    write(&folder_setup.dest_nested_file, "Dest nested file contents").unwrap();

    let file_actual = pull_tendril(
        &file_setup.tendrils_dir,
        &file_setup.tendril,
        true,
    );
    let folder_actual = pull_tendril(
        &folder_setup.tendrils_dir,
        &folder_setup.tendril,
        true,
    );

    assert!(matches!(file_actual, Err(TendrilActionError::Skipped)));
    assert!(matches!(folder_actual, Err(TendrilActionError::Skipped)));
    assert_eq!(file_setup.dest_file_contents(), "Dest file contents");
    assert_eq!(file_setup.tendrils_dir.join("SomeApp").read_dir().iter().count(), 1);
    assert_eq!(folder_setup.dest_nested_file_contents(), "Dest nested file contents");
    assert_eq!(folder_setup.tendrils_dir.join("SomeApp").read_dir().iter().count(), 1);
}

// TODO: Test when path is invalid and a copy is attempted (with both a folder AND a file)

#[rstest]
#[case("TendrilsFolder", "SomeApp", "<user>")]
#[case("TendrilsFolder", "<user>", "misc")]
#[case("<user>", "SomeApp", "misc")]
#[cfg(not(windows))] // These are invalid paths on Windows
fn supported_var_in_tendrils_folder_or_group_or_name_uses_raw_path(
    #[case] td_folder: &str,
    #[case] group: &str,
    #[case] name: &str
) {
    // TODO: This now should use raw paths (will be a Unix only test)
    let mut opts = SetupOpts::default();
    opts.tendrils_dirname = td_folder;
    opts.group = group;
    opts.source_filename = name;
    opts.make_source_folder = false;
    let setup = Setup::new(&opts);

    pull_tendril(&setup.tendrils_dir, &setup.tendril, false).unwrap();

    assert_eq!(setup.dest_file_contents(), "Source file contents");
    assert!(setup.tendrils_dir.join(group).read_dir().unwrap().count() == 1);
}

#[rstest]
#[case("Parent<>Folder")]
#[case("Parent<unsupported>Folder")]
#[case("<unsupported>")]
#[cfg(not(windows))] // These are invalid paths on Windows
fn unsupported_var_in_parent_path_uses_raw_path(#[case] parent_name_raw: &str) {
    let mut opts = SetupOpts::default();
    opts.parent_dirname = parent_name_raw;
    opts.make_source_folder = false;
    let setup = Setup::new(&opts);

    pull_tendril(&setup.tendrils_dir, &setup.tendril, false).unwrap();

    assert_eq!(setup.dest_file_contents(), "Source file contents");
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_doesnt_exist_returns_io_error_not_found(#[case] dry_run: bool) {
    let mut setup = Setup::new(&SetupOpts::default());
    setup.tendril.parent = PathBuf::from("SomePathThatDoesNotExist");

    let actual = pull_tendril(
        &setup.tendrils_dir,
        &setup.tendril,
        dry_run
    ).unwrap_err();
    match actual {
        TendrilActionError::IoError(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        _ => panic!(),
    }
    assert!(is_empty(&setup.tendrils_dir));
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_is_given_tendrils_folder_returns_recursion_error(#[case] dry_run: bool) {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let given_tendrils_folder = temp_parent_folder
        .path()
        .join("TendrilsFolder");
    let given_parent_folder = temp_parent_folder.path().to_path_buf();
    create_dir_all(&temp_parent_folder.path().join(get_username_can_panic())).unwrap();

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "TendrilsFolder".to_string(),
        given_parent_folder,
        TendrilMode::FolderOverwrite,
    ).unwrap();

    let actual = pull_tendril(&given_tendrils_folder, &given, dry_run);

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
    assert!(is_empty(&given_tendrils_folder));
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_is_ancestor_to_given_tendrils_folder_returns_recursion_error(#[case] dry_run: bool) {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let given_parent_folder = temp_parent_folder.path().to_path_buf();
    let given_tendrils_folder = given_parent_folder
        .join("Nested1")
        .join("Nested2")
        .join("Nested3")
        .join("TendrilsFolder");
    create_dir_all(
        &temp_parent_folder.path().join(get_username_can_panic())
    ).unwrap();

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "Nested1".to_string(),
        given_parent_folder,
        TendrilMode::FolderOverwrite,
    ).unwrap();

    let actual = pull_tendril(&given_tendrils_folder, &given, dry_run);

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
    assert!(is_empty(&given_tendrils_folder));
}

#[test]
fn source_is_sibling_to_given_tendrils_folder_copies_normally() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let given_parent_folder = temp_parent_folder.path().to_path_buf();
    let given_tendrils_folder = given_parent_folder
        .join("TendrilsFolder");
    create_dir_all(&given_parent_folder
        .join("SiblingFolder")
    ).unwrap();

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "SiblingFolder".to_string(),
        given_parent_folder,
        TendrilMode::FolderOverwrite
    ).unwrap();

    pull_tendril(&given_tendrils_folder, &given, false).unwrap();

    assert!(given_tendrils_folder
        .join("SomeApp")
        .join("SiblingFolder")
        .exists()
    );
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_is_direct_child_of_given_tendrils_folder_returns_recursion_error(#[case] dry_run: bool) {
    let temp_tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "TendrilsFolder"
    ).unwrap();
    let given_tendrils_folder = temp_tendrils_folder.path().to_path_buf();
    let given_parent_folder = given_tendrils_folder.clone();
    let source = given_tendrils_folder.join("misc.txt");
    create_dir_all(&given_tendrils_folder).unwrap();
    write(&source, "Source file contents").unwrap();

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        given_parent_folder,
        TendrilMode::FolderOverwrite,
    ).unwrap();

    let actual = pull_tendril(&given_tendrils_folder, &given, dry_run);

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
    assert_eq!(read_to_string(source).unwrap(), "Source file contents");
    assert!(given_tendrils_folder.read_dir().unwrap().into_iter().count() == 1);
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_is_nested_child_of_given_tendrils_folder_returns_recursion_error(#[case] dry_run: bool) {
    let temp_tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "TendrilsFolder"
    ).unwrap();
    let given_tendrils_folder = temp_tendrils_folder.path().to_path_buf();
    let given_parent_folder = given_tendrils_folder
        .join("Nested1")
        .join("Nested2")
        .join("Nested3");
    let source = given_parent_folder
        .join("misc.txt");
    create_dir_all(&source.parent().unwrap()).unwrap();
    write(&source, "Source file contents").unwrap();

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        given_parent_folder,
        TendrilMode::FolderOverwrite,
    ).unwrap();

    let actual = pull_tendril(&given_tendrils_folder, &given, dry_run);

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
    assert_eq!(read_to_string(source).unwrap(), "Source file contents");
    assert!(given_tendrils_folder.read_dir().unwrap().into_iter().count() == 1);
}

#[test]
fn source_is_another_tendrils_folder_still_copies() {
    let mut opts = SetupOpts::default();
    opts.is_folder_tendril = true;
    let setup = Setup::new(&opts);
    write(&setup.source_folder.join("tendrils.json"), "").unwrap();
    assert!(is_tendrils_folder(&setup.source_folder));

    pull_tendril(&setup.tendrils_dir, &setup.tendril, false).unwrap();

    assert!(setup.dest_folder.join("tendrils.json").exists());
    assert!(setup.dest_nested_file.exists());
    assert_eq!(setup.dest_folder.read_dir().unwrap().count(), 2);
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_is_file_and_dest_is_dir_returns_type_mismatch_error(#[case] dry_run: bool) {
    let mut opts = SetupOpts::default();
    opts.is_folder_tendril = false;
    let setup = Setup::new(&opts);
    create_dir_all(&setup.dest_file).unwrap();

    let actual = pull_tendril(
        &setup.tendrils_dir,
        &setup.tendril,
        dry_run
    ).unwrap_err();

    assert_eq!(setup.source_file_contents(), "Source file contents");
    assert!(matches!(actual, TendrilActionError::TypeMismatch));
    assert!(setup.source_file.is_file());
    assert!(setup.dest_file.is_dir());
    assert!(is_empty(&setup.dest_file));
    assert_eq!(setup.tendrils_dir.read_dir().iter().count(), 1);
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_is_dir_and_dest_is_file_returns_type_mismatch_error(#[case] dry_run: bool) {
    let mut opts = SetupOpts::default();
    opts.is_folder_tendril = true;
    let setup = Setup::new(&opts);
    create_dir_all(&setup.dest_folder.parent().unwrap()).unwrap();
    write(&setup.dest_folder, "I'm not a folder!").unwrap();

    let actual = pull_tendril(
        &setup.tendrils_dir,
        &setup.tendril,
        dry_run
    ).unwrap_err();

    let dest_file_contents = read_to_string(&setup.dest_folder).unwrap();
    assert_eq!(dest_file_contents, "I'm not a folder!");
    assert!(matches!(actual, TendrilActionError::TypeMismatch));
    assert!(setup.source_folder.is_dir());
    assert!(setup.dest_folder.is_file());
    assert_eq!(setup.source_folder.read_dir().iter().count(), 1);
    assert_eq!(setup.tendrils_dir.read_dir().iter().count(), 1);
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_is_symlink_returns_type_mismatch_error(#[case] dry_run: bool) {
    let temp_tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "TendrilsFolder"
    ).unwrap();

    let given_parent_folder = get_samples_folder()
        .join("SymlinksSource");

    let file_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "symfile.txt".to_string(),
        given_parent_folder.clone(),
        TendrilMode::FolderOverwrite,
    ).unwrap();
    let folder_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "symdir".to_string(),
        given_parent_folder,
        TendrilMode::FolderOverwrite,
    ).unwrap();

    let actual_1 = pull_tendril(
        &temp_tendrils_folder.path(),
        &file_tendril,
        dry_run,
    ).unwrap_err();
    let actual_2 = pull_tendril(
        &temp_tendrils_folder.path(),
        &folder_tendril,
        dry_run,
    ).unwrap_err();

    assert!(matches!(actual_1, TendrilActionError::TypeMismatch));
    assert!(matches!(actual_2, TendrilActionError::TypeMismatch));
    assert!(is_empty(&temp_tendrils_folder.path()));
}

#[rstest]
#[case(true)]
#[case(false)]
fn dest_is_symlink_returns_type_mismatch_error(#[case] dry_run: bool) {
    let temp_source_folder = TempDir::new_in(
        get_disposable_folder(),
        "SourceFolder"
    ).unwrap();

    let given_parent_folder = temp_source_folder.path().to_path_buf();
    let source_file = given_parent_folder
        .join("symfile.txt");
    let source_folder = given_parent_folder
        .join("symdir");
    let given_tendrils_folder = get_samples_folder().join("SymlinksDest");
    let dest_file = given_tendrils_folder.join("SomeApp").join("symfile.txt");
    let dest_folder = given_tendrils_folder.join("SomeApp").join("symdir");
    write(&source_file, "Source file contents").unwrap();
    create_dir_all(&source_folder).unwrap();

    let file_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "symfile.txt".to_string(),
        given_parent_folder.clone(),
        TendrilMode::FolderOverwrite,
    ).unwrap();
    let folder_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "symdir".to_string(),
        given_parent_folder,
        TendrilMode::FolderOverwrite,
    ).unwrap();

    let actual_1 = pull_tendril(
        &given_tendrils_folder,
        &file_tendril,
        dry_run,
    ).unwrap_err();
    let actual_2 = pull_tendril(
        &given_tendrils_folder,
        &folder_tendril,
        dry_run,
    ).unwrap_err();

    assert!(matches!(actual_1, TendrilActionError::TypeMismatch));
    assert!(matches!(actual_2, TendrilActionError::TypeMismatch));
    assert!(read_to_string(dest_file).unwrap().is_empty());
    assert!(read_to_string(dest_folder.join("misc.txt")).unwrap().is_empty())
}

#[test]
fn no_read_access_from_source_file_returns_io_error_permission_denied() {
    let temp_tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "TendrilsFolder"
    ).unwrap();

    // Note: This test sample is not version controlled and must first
    // be created using the setup script - See dev/setup-tendrils.nu
    let given_parent_folder = temp_tendrils_folder
        .path()
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .join("tests")
        .join("samples")
        .join("NoReadAccess");

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "no_read_access.txt".to_string(),
        given_parent_folder,
        TendrilMode::FolderOverwrite,
    ).unwrap();

    let actual = pull_tendril(&temp_tendrils_folder.path(), &given, false);

    match actual {
        Err(TendrilActionError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied)
        },
        _ => panic!()
    }
    assert!(is_empty(&temp_tendrils_folder.path().join("SomeApp")));
}

#[test]
fn no_read_access_from_source_folder_returns_io_error_permission_denied() {
    let temp_tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "TendrilsFolder"
    ).unwrap();
    let given_parent_folder = temp_tendrils_folder
        .path()
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .join("tests")
        .join("samples")
        .join("NoReadAccess");

    // Note: This test sample is not version controlled and must first
    // be created using the setup script - See dev/setup-tendrils.nu
    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "no_read_access_folder".to_string(),
        given_parent_folder,
        TendrilMode::FolderOverwrite,
    ).unwrap();

    let actual = pull_tendril(&temp_tendrils_folder.path(), &given, false);

    match actual {
        Err(TendrilActionError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied)
        },
        _ => panic!()
    }
    assert!(is_empty(&temp_tendrils_folder.path().join("SomeApp")));
}

#[test]
fn no_write_access_at_dest_file_returns_io_error_permission_denied() {
    let setup = Setup::new(&SetupOpts::default());
    create_dir_all(&setup.dest_file.parent().unwrap()).unwrap();
    write(&setup.dest_file, "Don't touch me").unwrap();

    // Set file read-only
    let mut perms = metadata(&setup.dest_file).unwrap().permissions();
    perms.set_readonly(true);
    set_permissions(&setup.dest_file, perms).unwrap();

    let actual = pull_tendril(&setup.tendrils_dir, &setup.tendril, false);

    match actual {
        Err(TendrilActionError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied)
        },
        _ => panic!()
    }
    assert_eq!(setup.dest_file_contents(), "Don't touch me");
}

// TODO: No write access at dest_folder?

#[rstest]
#[case(TendrilMode::FolderMerge)]
#[case(TendrilMode::FolderOverwrite)]
fn file_tendril_overwrites_dest_file_regardless_of_folder_merge_mode(
    #[case] mode: TendrilMode
) {
    let mut setup = Setup::new(&SetupOpts::default());
    create_dir_all(&setup.dest_file.parent().unwrap()).unwrap();
    write(&setup.dest_file, "Overwrite me!").unwrap();
    setup.tendril.mode = mode;

    pull_tendril(&setup.tendrils_dir, &setup.tendril, false).unwrap();

    assert_eq!(setup.dest_file_contents(), "Source file contents");
}

#[test]
fn folder_merge_false_w_folder_tendril_overwrites_dest_folder_recursively() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let given_tendrils_folder = &temp_parent_folder.path().join("TendrilsFolder");
    let source= &temp_parent_folder.path().join("SourceFolder");
    let nested_folder= &source.join("NestedFolder");
    let source_misc_file = source.join("misc.txt");
    let source_nested_file = nested_folder.join("nested.txt");
    let source_new_nested_file = nested_folder.join("new_nested.txt");
    let dest_misc_file = given_tendrils_folder
        .join("SomeApp")
        .join("SourceFolder")
        .join("misc.txt");
    let dest_nested_file = given_tendrils_folder
        .join("SomeApp")
        .join("SourceFolder")
        .join("NestedFolder")
        .join("nested.txt");
    let dest_new_nested_file = given_tendrils_folder
        .join("SomeApp")
        .join("SourceFolder")
        .join("NestedFolder")
        .join("new_nested.txt");
    let dest_extra_nested_file = given_tendrils_folder
        .join("SomeApp")
        .join("SourceFolder")
        .join("NestedFolder")
        .join("extra_nested.txt"); // Should no longer exist
    create_dir_all(&nested_folder).unwrap();
    create_dir_all(dest_nested_file.parent().unwrap()).unwrap();
    write(&source_misc_file, "Source misc file").unwrap();
    write(&source_nested_file, "Source nested file").unwrap();
    write(&source_new_nested_file, "I'm not in the tendrils folder").unwrap();
    write(&dest_misc_file, "Existing misc file").unwrap();
    write(&dest_nested_file, "Existing nested file").unwrap();
    write(&dest_extra_nested_file, "I'm not in the source folder").unwrap();

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "SourceFolder".to_string(),
        temp_parent_folder.path().to_path_buf(),
        TendrilMode::FolderOverwrite,
    ).unwrap();

    pull_tendril(&given_tendrils_folder, &given, false).unwrap();

    let dest_misc_contents = read_to_string(dest_misc_file).unwrap();
    let dest_nested_contents = read_to_string(dest_nested_file).unwrap();
    let dest_new_nested_contents = read_to_string(dest_new_nested_file).unwrap();
    assert_eq!(dest_misc_contents, "Source misc file");
    assert_eq!(dest_nested_contents, "Source nested file");
    assert_eq!(dest_new_nested_contents, "I'm not in the tendrils folder");
    assert!(!dest_extra_nested_file.exists());
}

#[test]
fn folder_merge_true_w_folder_tendril_merges_w_dest_folder_recursively() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let given_tendrils_folder = &temp_parent_folder.path().join("TendrilsFolder");
    let source= &temp_parent_folder.path().join("SourceFolder");
    let nested_folder= &source.join("NestedFolder");
    let source_misc_file = source.join("misc.txt");
    let source_nested_file = nested_folder.join("nested.txt");
    let source_new_nested_file = nested_folder.join("new_nested.txt");
    let dest_misc_file = given_tendrils_folder
        .join("SomeApp")
        .join("SourceFolder")
        .join("misc.txt");
    let dest_nested_file = given_tendrils_folder
        .join("SomeApp")
        .join("SourceFolder")
        .join("NestedFolder")
        .join("nested.txt");
    let dest_new_nested_file = given_tendrils_folder
        .join("SomeApp")
        .join("SourceFolder")
        .join("NestedFolder")
        .join("new_nested.txt");
    let dest_extra_nested_file = given_tendrils_folder
        .join("SomeApp")
        .join("SourceFolder")
        .join("NestedFolder")
        .join("extra_nested.txt");
    create_dir_all(&nested_folder).unwrap();
    create_dir_all(dest_nested_file.parent().unwrap()).unwrap();
    write(&source_misc_file, "Source misc file").unwrap();
    write(&source_nested_file, "Source nested file").unwrap();
    write(&source_new_nested_file, "I'm not in the tendrils folder").unwrap();
    write(&dest_misc_file, "Existing misc file").unwrap();
    write(&dest_nested_file, "Existing nested file").unwrap();
    write(&dest_extra_nested_file, "I'm not in the source folder").unwrap();

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "SourceFolder".to_string(),
        temp_parent_folder.path().to_path_buf(),
        TendrilMode::FolderMerge,
    ).unwrap();

    pull_tendril(&given_tendrils_folder, &given, false).unwrap();

    let dest_misc_contents = read_to_string(dest_misc_file).unwrap();
    let dest_nested_contents = read_to_string(dest_nested_file).unwrap();
    let dest_new_nested_contents = read_to_string(dest_new_nested_file).unwrap();
    let dest_extra_nested_contents = read_to_string(dest_extra_nested_file).unwrap();
    assert_eq!(dest_misc_contents, "Source misc file");
    assert_eq!(dest_nested_contents, "Source nested file");
    assert_eq!(dest_new_nested_contents, "I'm not in the tendrils folder");
    assert_eq!(dest_extra_nested_contents, "I'm not in the source folder");
}

#[test]
fn tendrils_folder_doesnt_exist_creates_folder_and_subfolders_first_except_if_dry_run() {
    let mut opts = SetupOpts::default();
    opts.make_tendrils_folder = false;
    let setup = Setup::new(&opts);

    let actual = pull_tendril(&setup.tendrils_dir, &setup.tendril, true);
    assert!(matches!(actual, Err(TendrilActionError::Skipped)));
    assert!(!setup.tendrils_dir.exists());

    pull_tendril(&setup.tendrils_dir, &setup.tendril, false).unwrap();
    assert_eq!(setup.dest_file_contents(), "Source file contents");
}

#[test]
fn file_tendril_source_is_unchanged() {
    let setup = Setup::new(&SetupOpts::default());

    pull_tendril(&setup.tendrils_dir, &setup.tendril, false).unwrap();

    assert_eq!(setup.source_file_contents(), "Source file contents");
    assert_eq!(setup.dest_file_contents(), "Source file contents");
}

#[test]
fn other_tendrils_in_same_group_folder_are_unchanged() {
    let setup = Setup::new(&SetupOpts::default());
    let some_other_tendril= &setup.tendrils_dir.join("SomeApp").join("other.txt");
    create_dir_all(setup.tendrils_dir.join("SomeApp")).unwrap();
    write(some_other_tendril, "Another tendril from the same app").unwrap();

    pull_tendril(&setup.tendrils_dir, &setup.tendril, false).unwrap();

    // Check that other tendril is unchanged
    let some_other_tendril_contents = read_to_string(some_other_tendril).unwrap();
    assert_eq!(some_other_tendril_contents, "Another tendril from the same app");
}

#[test]
fn folder_tendril_copies_all_contents_recursively_and_source_is_unchanged() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let given_tendrils_folder = &temp_parent_folder.path().join("TendrilsFolder");
    let source= &temp_parent_folder.path().join("SourceFolder");
    let nested_folder= &source.join("NestedFolder");
    create_dir_all(&nested_folder).unwrap();
    write(&source.join("misc.txt"), "Misc file contents").unwrap();
    write(&nested_folder.join("nested.txt"), "Nested file contents").unwrap();
    let dest_misc_file = given_tendrils_folder
        .join("SomeApp")
        .join("SourceFolder")
        .join("misc.txt");
    let dest_nested_file = given_tendrils_folder
        .join("SomeApp")
        .join("SourceFolder")
        .join("NestedFolder")
        .join("nested.txt");

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "SourceFolder".to_string(),
        temp_parent_folder.path().to_path_buf(),
        TendrilMode::FolderOverwrite,
    ).unwrap();

    pull_tendril(&given_tendrils_folder, &given, false).unwrap();

    let dest_misc_contents = read_to_string(dest_misc_file).unwrap();
    let dest_nested_contents = read_to_string(dest_nested_file).unwrap();
    assert_eq!(dest_misc_contents, "Misc file contents");
    assert_eq!(dest_nested_contents, "Nested file contents");

    // Check that source is unchanged
    let source_misc_contents = read_to_string(source.join("misc.txt")).unwrap();
    let source_nested_contents = read_to_string(nested_folder.join("nested.txt")).unwrap();
    assert_eq!(source_misc_contents, "Misc file contents");
    assert_eq!(source_nested_contents, "Nested file contents");
}

#[rstest]
#[case(true)]
#[case(false)]
fn given_link_mode_tendril_returns_mode_mismatch_error(#[case] dry_run: bool) {
    let mut setup = Setup::new(&SetupOpts::default());
    setup.tendril.mode = TendrilMode::Link;

    let actual = pull_tendril(&setup.tendrils_dir, &setup.tendril, dry_run);

    assert!(matches!(actual, Err(TendrilActionError::ModeMismatch)));
    assert_eq!(setup.source_file_contents(), "Source file contents");
    assert!(is_empty(&setup.tendrils_dir));
}
