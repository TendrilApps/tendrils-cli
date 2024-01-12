use crate::{
    is_tendrils_folder,
    pull_tendril,
    PushPullError,
    Tendril};
use crate::test_utils::{
    get_disposable_folder,
    get_samples_folder,
    get_username_can_panic,
    is_empty,
    set_all_platform_paths,
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

#[test]
fn parent_path_list_is_empty_returns_skipped_error() {
    let mut setup = Setup::new(&SetupOpts::default());
    set_all_platform_paths(&mut setup.tendril, &[].to_vec());

    let actual = pull_tendril(&setup.tendrils_dir, &setup.tendril).unwrap_err();

    assert!(matches!(actual, PushPullError::Skipped));
    assert!(is_empty(&setup.tendrils_dir));
}

#[test]
fn parent_path_is_empty_string_attempts_copy() {
    let mut setup = Setup::new(&SetupOpts::default());
    setup.tendril.name = "SomeNameThatDefinitelyDoesn'tExist.txt".to_string();
    set_all_platform_paths(&mut setup.tendril, &[PathBuf::from("")].to_vec());

    let actual = pull_tendril(&setup.tendrils_dir, &setup.tendril).unwrap_err();

    match actual {
        PushPullError::IoError(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        _ => panic!("Wrong error type")
    }
    assert!(is_empty(&setup.tendrils_dir));
}

#[rstest]
#[case("NoDot")]
#[case("single.dot")]
#[case("multi.sandwiched.dots")]
#[case(".LeadingDot")]
#[case("TrailingDot.")]
fn tendril_name_is_valid_copies_successfully(#[case] name: String) {
    let mut opts = SetupOpts::default();
    opts.source_filename = &name;
    opts.source_foldername = &name;
    opts.make_source_folder = false;
    let file_setup = Setup::new(&opts);
    opts.is_folder_tendril = true;
    opts.make_source_file = false;
    opts.make_source_folder = true;
    let folder_setup = Setup::new(&opts);

    pull_tendril(&file_setup.tendrils_dir, &file_setup.tendril).unwrap();
    pull_tendril(&folder_setup.tendrils_dir, &folder_setup.tendril).unwrap();
    
    assert_eq!(file_setup.dest_file_contents(), "Source file contents");
    assert_eq!(file_setup.tendrils_dir.join("SomeApp").read_dir().iter().count(), 1);
    assert_eq!(folder_setup.dest_nested_file_contents(), "Nested file contents");
    assert_eq!(folder_setup.tendrils_dir.join("SomeApp").read_dir().iter().count(), 1);
}

// TODO: Add tests for valid app field?

#[rstest]
#[case("TendrilsFolder", "SomeApp", "<user>")]
#[case("TendrilsFolder", "<user>", "misc")]
#[case("<user>", "SomeApp", "misc")]
#[cfg(not(windows))] // These are invalid paths on Windows
fn supported_var_in_tendrils_folder_or_app_or_name_uses_raw_path(
    #[case] td_folder: &str,
    #[case] app: &str,
    #[case] name: &str
) {
    let mut opts = SetupOpts::default();
    opts.tendrils_dirname = td_folder;
    opts.app = app;
    opts.source_filename = name;
    opts.make_source_folder = false;
    let setup = Setup::new(&opts);

    pull_tendril(&setup.tendrils_dir, &setup.tendril).unwrap();

    assert_eq!(setup.dest_file_contents(), "Source file contents");
    assert!(setup.tendrils_dir.join(app).read_dir().unwrap().count() == 1);
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

    pull_tendril(&setup.tendrils_dir, &setup.tendril).unwrap();

    assert_eq!(setup.dest_file_contents(), "Source file contents");
}

#[rstest]
#[case("<user>",                &get_username_can_panic())]
#[case("<user>LeadingVar",      &format!("{}LeadingVar", get_username_can_panic()))]
#[case("Sandwiched<user>Var",   &format!("Sandwiched{}Var", get_username_can_panic()))]
#[case("TrailingVar<user>",     &format!("TrailingVar{}", get_username_can_panic()))]
fn supported_var_in_parent_path_is_resolved(
    #[case] parent_name_raw: &str,
    #[case] parent_name_resolved: &str
) {
    let mut opts = SetupOpts::default();
    opts.parent_dirname = parent_name_resolved;
    opts.make_source_folder = false;
    opts.parent_dir_is_child_of_temp_dir = true;
    let mut setup = Setup::new(&opts);
    set_all_platform_paths(
        &mut setup.tendril,
        &[setup.temp_dir.path().join(parent_name_raw)]);

    pull_tendril(&setup.tendrils_dir, &setup.tendril).unwrap();

    assert_eq!(setup.dest_file_contents(), "Source file contents");
}

#[rstest]
#[case("<<user>>", &format!("<{}>", get_username()))]
#[cfg(not(windows))] // These are invalid paths on Windows
fn supported_var_in_parent_path_is_resolved_non_windows(
    #[case] parent_name_raw: &str,
    #[case] parent_name_resolved: &str
) {
    supported_var_in_parent_path_is_resolved(parent_name_raw, parent_name_resolved);
}

#[test]
fn resolved_source_path_doesnt_exist_returns_io_error_not_found() {
    let mut setup = Setup::new(&SetupOpts::default());
    set_all_platform_paths(
        &mut setup.tendril,
        &[PathBuf::from("SomePathThatDoesNotExist")].to_vec()
    );

    let actual = pull_tendril(&setup.tendrils_dir, &setup.tendril).unwrap_err();
    match actual {
        PushPullError::IoError(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        _ => panic!(),
    }
    assert!(is_empty(&setup.tendrils_dir));
}

#[test]
fn resolved_source_path_is_given_tendrils_folder_returns_recursion_error() {
    let temp_grandparent_folder = TempDir::new_in(
        get_disposable_folder(),
        "GrandparentFolder"
    ).unwrap();
    let given_parent_folder = temp_grandparent_folder.path().join("<user>");
    let given_tendrils_folder = temp_grandparent_folder
        .path()
        .join(get_username_can_panic())
        .join("TendrilsFolder");
    create_dir_all(&temp_grandparent_folder.path().join(get_username_can_panic())).unwrap();

    let mut given = Tendril::new("SomeApp", "TendrilsFolder");
    set_all_platform_paths(&mut given, &[given_parent_folder]);

    let actual = pull_tendril(&given_tendrils_folder, &given);

    assert!(matches!(actual, Err(PushPullError::Recursion)));
    assert!(is_empty(&given_tendrils_folder));
}

#[test]
fn resolved_source_path_is_ancestor_to_given_tendrils_folder_returns_recursion_error() {
    let temp_grandparent_folder = TempDir::new_in(
        get_disposable_folder(),
        "GrandparentFolder"
    ).unwrap();
    let given_parent_folder = temp_grandparent_folder.path().join("<user>");
    let given_tendrils_folder = temp_grandparent_folder
        .path()
        .join(get_username_can_panic())
        .join("Nested1")
        .join("Nested2")
        .join("Nested3")
        .join("TendrilsFolder");
    create_dir_all(
        &temp_grandparent_folder.path().join(get_username_can_panic())
    ).unwrap();

    let mut given = Tendril::new("SomeApp", "Nested1");
    set_all_platform_paths(&mut given, &[given_parent_folder]);

    let actual = pull_tendril(&given_tendrils_folder, &given);

    assert!(matches!(actual, Err(PushPullError::Recursion)));
    assert!(is_empty(&given_tendrils_folder));
}

#[test]
fn resolved_source_path_is_sibling_to_given_tendrils_folder_copies_normally() {
    let temp_grandparent_folder = TempDir::new_in(
        get_disposable_folder(),
        "GrandparentFolder"
    ).unwrap();
    let given_parent_folder = temp_grandparent_folder.path().join("<user>");
    let given_tendrils_folder = temp_grandparent_folder
        .path()
        .join(get_username_can_panic())
        .join("TendrilsFolder");
    create_dir_all(&temp_grandparent_folder
        .path()
        .join(get_username_can_panic())
        .join("SiblingFolder")
    ).unwrap();

    let mut given = Tendril::new("SomeApp", "SiblingFolder");
    set_all_platform_paths(&mut given, &[given_parent_folder]);

    pull_tendril(&given_tendrils_folder, &given).unwrap();

    assert!(given_tendrils_folder
        .join("SomeApp")
        .join("SiblingFolder")
        .exists()
    );
}

#[test]
fn resolved_source_path_is_direct_child_of_given_tendrils_folder_returns_recursion_error() {
    let temp_grandparent_folder = TempDir::new_in(
        get_disposable_folder(),
        "GrandparentFolder"
    ).unwrap();
    let given_tendrils_folder = temp_grandparent_folder
        .path()
        .join(get_username_can_panic())
        .join("TendrilsFolder");
    let given_parent_folder = temp_grandparent_folder
        .path()
        .join("<user>")
        .join("TendrilsFolder");
    let source = given_tendrils_folder.join("misc.txt");
    create_dir_all(&given_tendrils_folder).unwrap();
    write(&source, "Source file contents").unwrap();

    let mut given = Tendril::new("SomeApp", "misc.txt");
    set_all_platform_paths(&mut given, &[given_parent_folder]);

    let actual = pull_tendril(&given_tendrils_folder, &given);

    assert!(matches!(actual, Err(PushPullError::Recursion)));
    assert_eq!(read_to_string(source).unwrap(), "Source file contents");
    assert!(given_tendrils_folder.read_dir().unwrap().into_iter().count() == 1);
}

#[test]
fn resolved_source_path_is_nested_child_of_given_tendrils_folder_returns_recursion_error() {
    let temp_grandparent_folder = TempDir::new_in(
        get_disposable_folder(),
        "GrandparentFolder"
    ).unwrap();
    let given_tendrils_folder = temp_grandparent_folder
        .path()
        .join(get_username_can_panic())
        .join("TendrilsFolder");
    let given_parent_folder = temp_grandparent_folder
        .path()
        .join("<user>")
        .join("TendrilsFolder")
        .join("Nested1")
        .join("Nested2")
        .join("Nested3");
    let source = given_tendrils_folder
        .join("Nested1")
        .join("Nested2")
        .join("Nested3")
        .join("misc.txt");
    create_dir_all(&source.parent().unwrap()).unwrap();
    write(&source, "Source file contents").unwrap();

    let mut given = Tendril::new("SomeApp", "misc.txt");
    set_all_platform_paths(&mut given, &[given_parent_folder]);

    let actual = pull_tendril(&given_tendrils_folder, &given);

    assert!(matches!(actual, Err(PushPullError::Recursion)));
    assert_eq!(read_to_string(source).unwrap(), "Source file contents");
    assert!(given_tendrils_folder.read_dir().unwrap().into_iter().count() == 1);
}

#[test]
fn resolved_source_path_is_another_tendrils_folder_still_copies() {
    let mut opts = SetupOpts::default();
    opts.is_folder_tendril = true;
    let setup = Setup::new(&opts);
    write(&setup.source_folder.join("tendrils.json"), "").unwrap();
    assert!(is_tendrils_folder(&setup.source_folder));

    pull_tendril(&setup.tendrils_dir, &setup.tendril).unwrap();

    assert!(setup.dest_folder.join("tendrils.json").exists());
    assert!(setup.dest_nested_file.exists());
    assert_eq!(setup.dest_folder.read_dir().unwrap().count(), 2);
}

#[test]
fn resolved_source_path_is_file_and_dest_is_dir_returns_type_mismatch_error() {
    let mut opts = SetupOpts::default();
    opts.is_folder_tendril = false;
    let setup = Setup::new(&opts);
    create_dir_all(&setup.dest_file).unwrap();

    let actual = pull_tendril(&setup.tendrils_dir, &setup.tendril).unwrap_err();

    assert_eq!(setup.source_file_contents(), "Source file contents");
    assert!(matches!(actual, PushPullError::TypeMismatch));
    assert!(setup.source_file.is_file());
    assert!(setup.dest_file.is_dir());
    assert!(is_empty(&setup.dest_file));
    assert_eq!(setup.tendrils_dir.read_dir().iter().count(), 1);
}

#[test]
fn resolved_source_path_is_dir_and_dest_is_file_returns_type_mismatch_error() {
    let mut opts = SetupOpts::default();
    opts.is_folder_tendril = true;
    let setup = Setup::new(&opts);
    create_dir_all(&setup.dest_folder.parent().unwrap()).unwrap();
    write(&setup.dest_folder, "I'm not a folder!").unwrap();

    let actual = pull_tendril(&setup.tendrils_dir, &setup.tendril).unwrap_err();

    let dest_file_contents = read_to_string(&setup.dest_folder).unwrap();
    assert_eq!(dest_file_contents, "I'm not a folder!");
    assert!(matches!(actual, PushPullError::TypeMismatch));
    assert!(setup.source_folder.is_dir());
    assert!(setup.dest_folder.is_file());
    assert_eq!(setup.source_folder.read_dir().iter().count(), 1);
    assert_eq!(setup.tendrils_dir.read_dir().iter().count(), 1);
}

#[test]
fn resolved_source_path_is_symlink_returns_type_mismatch_error() {
    let temp_tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "TendrilsFolder"
    ).unwrap();

    let given_parent_folder = get_samples_folder()
        .join("SymlinksSource");
    let source_file = given_parent_folder.join("symfile.txt");
    let source_folder = given_parent_folder.join("symdir");

    let mut file_tendril = Tendril::new("SomeApp", "symfile.txt");
    let mut folder_tendril = Tendril::new("SomeApp", "symdir");
    set_all_platform_paths(
        &mut file_tendril,
        &[source_file.parent().unwrap().to_path_buf()]
    );
    set_all_platform_paths(
        &mut folder_tendril,
        &[source_folder.parent().unwrap().to_path_buf()]
    );

    let actual_1 = pull_tendril(
        &temp_tendrils_folder.path(),
        &file_tendril
    ).unwrap_err();
    let actual_2 = pull_tendril(
        &temp_tendrils_folder.path(),
        &folder_tendril
    ).unwrap_err();

    assert!(matches!(actual_1, PushPullError::TypeMismatch));
    assert!(matches!(actual_2, PushPullError::TypeMismatch));
    assert!(is_empty(&temp_tendrils_folder.path()));
}

#[test]
fn dest_is_symlink_returns_type_mismatch_error() {
    let temp_source_folder = TempDir::new_in(
        get_disposable_folder(),
        "SourceFolder"
    ).unwrap();

    let given_parent_folder = temp_source_folder.path();
    let source_file = given_parent_folder
        .join("symfile.txt");
    let source_folder = given_parent_folder
        .join("symdir");
    let given_tendrils_folder = get_samples_folder().join("SymlinksDest");
    let dest_file = given_tendrils_folder.join("SomeApp").join("symfile.txt");
    let dest_folder = given_tendrils_folder.join("SomeApp").join("symdir");
    write(&source_file, "Source file contents").unwrap();
    create_dir_all(&source_folder).unwrap();

    let mut file_tendril = Tendril::new("SomeApp", "symfile.txt");
    let mut folder_tendril = Tendril::new("SomeApp", "symdir");
    set_all_platform_paths(&mut file_tendril, &[given_parent_folder.to_path_buf()]);
    set_all_platform_paths(&mut folder_tendril, &[given_parent_folder.to_path_buf()]);

    let actual_1 = pull_tendril(
        &given_tendrils_folder,
        &file_tendril
    ).unwrap_err();
    let actual_2 = pull_tendril(
        &given_tendrils_folder,
        &folder_tendril
    ).unwrap_err();

    assert!(matches!(actual_1, PushPullError::TypeMismatch));
    assert!(matches!(actual_2, PushPullError::TypeMismatch));
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
    let source = &temp_tendrils_folder
        .path()
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .join("tests")
        .join("samples")
        .join("NoReadAccess")
        .join("no_read_access.txt");

    let mut given = Tendril::new("SomeApp", "no_read_access.txt");
    set_all_platform_paths(&mut given, &[source.parent().unwrap().to_path_buf()]);

    let actual = pull_tendril(&temp_tendrils_folder.path(), &given);

    match actual {
        Err(PushPullError::IoError(e)) => {
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
    // Note: This test sample is not version controlled and must first
    // be created using the setup script - See dev/setup-tendrils.nu
    let source = &temp_tendrils_folder
        .path()
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .join("tests")
        .join("samples")
        .join("NoReadAccess")
        .join("no_read_access_folder");

    let mut given = Tendril::new("SomeApp", "no_read_access_folder");
    set_all_platform_paths(&mut given, &[source.parent().unwrap().to_owned()]);

    let actual = pull_tendril(&temp_tendrils_folder.path(), &given);

    match actual {
        Err(PushPullError::IoError(e)) => {
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

    let actual = pull_tendril(&setup.tendrils_dir, &setup.tendril);

    match actual {
        Err(PushPullError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied)
        },
        _ => panic!()
    }
    assert_eq!(setup.dest_file_contents(), "Don't touch me");
}

// TODO: No write access at dest_folder?

#[rstest]
#[case(true)]
#[case(false)]
fn file_tendril_overwrites_dest_file_regardless_of_folder_merge(
    #[case] folder_merge: bool
) {
    let mut setup = Setup::new(&SetupOpts::default());
    create_dir_all(&setup.dest_file.parent().unwrap()).unwrap();
    write(&setup.dest_file, "Overwrite me!").unwrap();
    setup.tendril.folder_merge = folder_merge;

    pull_tendril(&setup.tendrils_dir, &setup.tendril).unwrap();

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

    let mut given = Tendril::new("SomeApp", "SourceFolder");
    given.folder_merge = false;
    set_all_platform_paths(&mut given, &[temp_parent_folder.path().to_path_buf()]);

    pull_tendril(&given_tendrils_folder, &given).unwrap();

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

    let mut given = Tendril::new("SomeApp", "SourceFolder");
    given.folder_merge = true;
    set_all_platform_paths(&mut given, &[temp_parent_folder.path().to_path_buf()]);

    pull_tendril(&given_tendrils_folder, &given).unwrap();

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
fn tendrils_folder_doesnt_exist_creates_folder_and_subfolders_first() {
    let mut opts = SetupOpts::default();
    opts.make_tendrils_folder = false;
    let setup = Setup::new(&opts);

    pull_tendril(&setup.tendrils_dir, &setup.tendril).unwrap();

    assert_eq!(setup.dest_file_contents(), "Source file contents");
}

#[test]
fn file_tendril_source_is_unchanged() {
    let setup = Setup::new(&SetupOpts::default());

    pull_tendril(&setup.tendrils_dir, &setup.tendril).unwrap();

    assert_eq!(setup.source_file_contents(), "Source file contents");
    assert_eq!(setup.dest_file_contents(), "Source file contents");
}

#[test]
fn other_tendrils_in_same_app_folder_are_unchanged() {
    let setup = Setup::new(&SetupOpts::default());
    let some_other_tendril= &setup.tendrils_dir.join("SomeApp").join("other.txt");
    create_dir_all(setup.tendrils_dir.join("SomeApp")).unwrap();
    write(some_other_tendril, "Another tendril from the same app").unwrap();

    pull_tendril(&setup.tendrils_dir, &setup.tendril).unwrap();

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

    let mut given = Tendril::new("SomeApp", "SourceFolder");
    set_all_platform_paths(
        &mut given,
        &[temp_parent_folder.path().to_path_buf()]
    );

    pull_tendril(&given_tendrils_folder, &given).unwrap();

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

#[test]
fn copies_from_correct_platform_paths() {
    let temp_grandparent_folder = TempDir::new_in(
        get_disposable_folder(),
        "GrandparentFolder"
    ).unwrap();
    let given_tendrils_folder = &temp_grandparent_folder.path().join("TendrilsFolder");
    let parent_mac = temp_grandparent_folder.path().join("Mac");
    let parent_win = temp_grandparent_folder.path().join("Windows");
    let source_mac= parent_mac.join("misc.txt");
    let source_win= parent_win.join("misc.txt");
    let dest = given_tendrils_folder.join("SomeApp").join("misc.txt");
    create_dir_all(&parent_mac).unwrap();
    create_dir_all(&parent_win).unwrap();
    write(source_mac, "Mac file contents").unwrap();
    write(source_win, "Windows file contents").unwrap();

    let mut given = Tendril::new("SomeApp", "misc.txt");
    given.parent_dirs_mac = [parent_mac.to_str().unwrap().to_string()].to_vec();
    given.parent_dirs_windows = [parent_win.to_str().unwrap().to_string()].to_vec();

    pull_tendril(given_tendrils_folder, &given).unwrap();

    let dest_contents = read_to_string(dest).unwrap();
    match std::env::consts::OS {
        "macos" => assert_eq!(dest_contents, "Mac file contents"),
        "windows" => assert_eq!(dest_contents, "Windows file contents"),
        _ => unimplemented!()
    }
}

#[test]
fn multiple_paths_only_copies_first() {
    let temp_grandparent_folder = TempDir::new_in(
        get_disposable_folder(),
        "GrandparentFolder"
    ).unwrap();
    let given_tendrils_folder = temp_grandparent_folder.path().join("TendrilsFolder");
    let given_parent_folder_1 = temp_grandparent_folder.path().join("Parent1");
    let given_parent_folder_2 = temp_grandparent_folder.path().join("Parent2");
    create_dir_all(&given_tendrils_folder).unwrap();
    create_dir_all(&given_parent_folder_1).unwrap();
    create_dir_all(&given_parent_folder_2).unwrap();
    write(given_parent_folder_1.join("misc.txt"), "Copy me!").unwrap();
    write(given_parent_folder_2.join("misc.txt"), "Don't copy me!").unwrap();

    let mut tendril = Tendril::new("SomeApp", "misc.txt");
    set_all_platform_paths(&mut tendril, &[
        given_parent_folder_1,
        given_parent_folder_2.clone(),
        given_parent_folder_2, // Duplicate
        PathBuf::from("I_Do_Not_Exist")
    ]);

    pull_tendril(&given_tendrils_folder, &tendril).unwrap();

    let dest_file_contents = read_to_string(
        given_tendrils_folder.join("SomeApp").join("misc.txt")
    ).unwrap();
    assert_eq!(dest_file_contents, "Copy me!");
    assert!(given_tendrils_folder.join("SomeApp").read_dir().unwrap().count() == 1);
}

#[test]
fn multiple_paths_first_is_missing_returns_not_found_error() {
    let mut setup = Setup::new(&SetupOpts::default());

    set_all_platform_paths(
        &mut setup.tendril,
        &[PathBuf::from("I_Do_Not_Exist"), setup.parent_dir]
    );

    let actual = pull_tendril(&setup.tendrils_dir, &setup.tendril);

    match actual {
        Err(PushPullError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound)
        },
        _ => panic!(),
    }
    assert!(is_empty(&setup.tendrils_dir));
}
