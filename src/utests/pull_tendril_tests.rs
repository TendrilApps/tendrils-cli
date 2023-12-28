use crate::{
    is_tendrils_folder,
    pull_tendril,
    PushPullError,
    Tendril};
use crate::utests::common::{
    get_disposable_folder,
    get_username,
    is_empty,
    set_all_platform_paths,
};
use rstest::rstest;
use serial_test::serial;
use std::fs::{
    create_dir_all,
    File,
    metadata,
    read_to_string,
    remove_dir_all,
    remove_file,
    set_permissions,
    write,
};
use std::path::PathBuf;
use tempdir::TempDir;

// TODO: Create a get_context and a context stuct that sets up
// the folder and returns references to the parent, source, dest, given tendril, etc
// Implement drop on the struct so that it can be deleted

#[test]
fn parent_path_list_is_empty_returns_skipped_error() {
    let temp_tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "ShouldBeEmpty"
    ).unwrap().into_path();

    let mut given = Tendril::new("SomeApp", "misc.txt");
    set_all_platform_paths(
        &mut given,
        &[].to_vec()
    );

    let actual = pull_tendril(&temp_tendrils_folder, &given).unwrap_err();

    assert!(matches!(actual, PushPullError::Skipped));
    assert!(is_empty(&temp_tendrils_folder));
}

#[test]
fn parent_path_is_empty_string_attempts_copy() {
    let temp_tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "TendrilsFolder"
    ).unwrap().into_path();

    let mut given =
        Tendril::new("SomeApp", "SomeNameThatDefinitelyDoesn'tExist.txt");
    set_all_platform_paths(
        &mut given,
        &[PathBuf::from("")].to_vec()
    );

    let actual = pull_tendril(&temp_tendrils_folder, &given).unwrap_err();

    match actual {
        PushPullError::IoError(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        _ => panic!("Wrong error type")
    }
    assert!(is_empty(&temp_tendrils_folder));
}

#[rstest]
#[case("")]
#[case("some/path")]
#[case("some\\path")]
#[case(".git")] // TODO: Test that other variations that contain git
                // are allowed (.gitsomething, git, something.git)
#[case(".Git")]
#[case(".GIT")]
fn tendril_app_is_invalid_returns_invalid_id_error(#[case] app: &str) {
    let temp_tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "TendrilsFolder"
    ).unwrap().into_path();

    let given = Tendril::new(&app, "misc");

    let actual = pull_tendril(&temp_tendrils_folder, &given);

    assert!(matches!(actual, Err(PushPullError::InvalidId)));
    assert!(is_empty(&temp_tendrils_folder))
}

#[rstest]
#[case("")]
#[case("some/path")]
#[case("some\\path")]
fn tendril_name_is_invalid_returns_invalid_id_error(#[case] name: &str) {
    let temp_tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "TendrilsFolder"
    ).unwrap().into_path();

    let given = Tendril::new("SomeApp", name);

    let actual = pull_tendril(&temp_tendrils_folder, &given);

    assert!(matches!(actual, Err(PushPullError::InvalidId)));
    assert!(is_empty(&temp_tendrils_folder))
}

#[rstest]
#[case("NoDot")]
#[case("single.dot")]
#[case("multi.sandwiched.dots")]
#[case(".LeadingDot")]
#[case("TrailingDot.")]
fn tendril_name_is_valid_copies_successfully(#[case] name: &str) {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = temp_parent_folder.join("TendrilsFolder");
    let source_file = temp_parent_folder.join(name);
    let source_folder = temp_parent_folder.join(name);
    let source_nested_file = source_folder.join("nested.txt");
    let dest_file = given_tendrils_folder.join("SomeApp").join(name);
    let dest_folder = given_tendrils_folder.join("SomeApp").join(name);
    let dest_nested_file = dest_folder.join("nested.txt");
    let mut given_file_tendril = Tendril::new("SomeApp", name);
    let mut given_folder_tendril = Tendril::new("SomeApp", name);
    set_all_platform_paths(&mut given_file_tendril, &[temp_parent_folder.clone()]);
    set_all_platform_paths(&mut given_folder_tendril, &[temp_parent_folder]);

    write(&source_file, "Source file contents").unwrap();
    pull_tendril(&given_tendrils_folder, &given_file_tendril).unwrap();
    let dest_file_contents = read_to_string(&dest_file).unwrap();
    assert_eq!(dest_file_contents, "Source file contents");
    assert_eq!(given_tendrils_folder.join("SomeApp").read_dir().iter().count(), 1);
    remove_file(source_file).unwrap();
    remove_dir_all(&given_tendrils_folder).unwrap();

    create_dir_all(&source_folder).unwrap();
    write(source_nested_file, "Source nested file contents").unwrap();
    pull_tendril(&given_tendrils_folder, &given_folder_tendril).unwrap();
    let dest_file_contents = read_to_string(&dest_nested_file).unwrap();
    assert_eq!(dest_file_contents, "Source nested file contents");
    assert_eq!(given_tendrils_folder.join("SomeApp").read_dir().iter().count(), 1);
    assert_eq!(dest_folder.read_dir().iter().count(), 1);
}

// TODO: Add tests for valid app field?

#[rstest]
#[case("TendrilsFolder", "SomeApp", "<user>")]
#[case("TendrilsFolder", "<user>", "misc")]
#[case("<user>", "SomeApp", "misc")]
fn supported_var_in_tendrils_folder_or_app_or_name_uses_raw_path(
    #[case] td_folder: &str,
    #[case] app: &str,
    #[case] name: &str
) {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = temp_parent_folder.join(td_folder);
    let source = temp_parent_folder.join(name);
    let dest = given_tendrils_folder.join(app).join(name);
    File::create(&temp_parent_folder.join(name)).unwrap();
    write(source, "Source file contents").unwrap();

    let mut given = Tendril::new(app, name);
    set_all_platform_paths(&mut given, &[temp_parent_folder]);

    pull_tendril(&given_tendrils_folder, &given).unwrap();

    let dest_file_contents = read_to_string(dest).unwrap();
    assert_eq!(dest_file_contents, "Source file contents");
    assert!(given_tendrils_folder.join(app).read_dir().unwrap().count() == 1);
}

#[rstest]
#[case("Parent<>Folder")]
#[case("Parent<unsupported>Folder")]
#[case("<unsupported>")]
fn unsupported_var_in_parent_path_uses_raw_path(#[case] parent_name_raw: &str) {
    let temp_grandparent_folder = TempDir::new_in(
        get_disposable_folder(),
        "GrandparentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = 
        temp_grandparent_folder.join("TendrilsFolder");
    let source = temp_grandparent_folder.join(parent_name_raw).join("misc.txt");
    let dest = given_tendrils_folder.join("SomeApp").join("misc.txt");
    create_dir_all(temp_grandparent_folder.join(parent_name_raw)).unwrap();
    write(source, "Source file contents").unwrap();

    let mut given = Tendril::new("SomeApp", "misc.txt");
    set_all_platform_paths(
        &mut given,
        &[temp_grandparent_folder.join(parent_name_raw)]
    );

    pull_tendril(&given_tendrils_folder, &given).unwrap();

    let dest_file_contents = read_to_string(dest).unwrap();
    assert_eq!(dest_file_contents, "Source file contents");
}

#[rstest]
#[case("<user>",                get_username())]
#[case("<user>LeadingVar",      format!("{}LeadingVar", get_username()))]
#[case("Sandwiched<user>Var",   format!("Sandwiched{}Var", get_username()))]
#[case("TrailingVar<user>",     format!("TrailingVar{}", get_username()))]
#[case("<<user>>",              format!("<{}>", get_username()))]
#[serial]
fn supported_var_in_parent_path_is_resolved(
    #[case] parent_name_raw: &str,
    #[case] parent_name_resolved: String
) {
    let temp_grandparent_folder = TempDir::new_in(
        get_disposable_folder(),
        "GrandparentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = &temp_grandparent_folder.join("TendrilsFolder");
    let source = 
        &temp_grandparent_folder.join(parent_name_resolved).join("misc.txt");
    let dest = &given_tendrils_folder.join("SomeApp").join("misc.txt");
    create_dir_all(&source.parent().unwrap()).unwrap();
    write(&source, "Source file contents").unwrap();

    let mut given = Tendril::new("SomeApp", "misc.txt");
    set_all_platform_paths(
        &mut given,
        &[temp_grandparent_folder.join(parent_name_raw)]
    );

    pull_tendril(&given_tendrils_folder, &given).unwrap();

    let dest_file_contents = read_to_string(&dest).unwrap();
    assert_eq!(dest_file_contents, "Source file contents");
}

#[test]
fn resolved_source_path_doesnt_exist_returns_io_error_not_found() {
    let temp_tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "ShouldBeEmpty"
    ).unwrap().into_path();

    let mut given = Tendril::new("SomeApp", "misc.txt");
    set_all_platform_paths(
        &mut given,
        &[PathBuf::from("SomePathThatDoesNotExist")].to_vec()
    );

    let actual = pull_tendril(&temp_tendrils_folder, &given).unwrap_err();
    match actual {
        PushPullError::IoError(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        _ => panic!("Wrong error kind"),
    }

    assert!(is_empty(&temp_tendrils_folder));
}

#[test]
fn resolved_source_path_is_given_tendrils_folder_returns_recursion_error() {
    let temp_grandparent_folder = TempDir::new_in(
        get_disposable_folder(),
        "GrandparentFolder"
    ).unwrap().into_path();
    let given_parent_folder = temp_grandparent_folder.join("<user>");
    let given_tendrils_folder = temp_grandparent_folder
        .join(get_username())
        .join("TendrilsFolder");
    create_dir_all(&temp_grandparent_folder.join(get_username())).unwrap();

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
    ).unwrap().into_path();
    let given_parent_folder = temp_grandparent_folder.join("<user>");
    let given_tendrils_folder = temp_grandparent_folder
        .join(get_username())
        .join("Nested1")
        .join("Nested2")
        .join("Nested3")
        .join("TendrilsFolder");
    create_dir_all(&temp_grandparent_folder.join(get_username())).unwrap();

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
    ).unwrap().into_path();
    let given_parent_folder = temp_grandparent_folder.join("<user>");
    let given_tendrils_folder = temp_grandparent_folder
        .join(get_username())
        .join("TendrilsFolder");
    create_dir_all(&temp_grandparent_folder
        .join(get_username())
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
    ).unwrap().into_path();
    let given_tendrils_folder = temp_grandparent_folder
        .join(get_username())
        .join("TendrilsFolder");
    let given_parent_folder = temp_grandparent_folder
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
    ).unwrap().into_path();
    let given_tendrils_folder = temp_grandparent_folder
        .join(get_username())
        .join("TendrilsFolder");
    let given_parent_folder = temp_grandparent_folder
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
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
    let source = &temp_parent_folder.join("AnotherTendrilsFolder");
    create_dir_all(&source).unwrap();
    write(&source.join("tendrils.json"), "").unwrap();
    assert!(is_tendrils_folder(&source));

    let mut given = Tendril::new("SomeApp", "AnotherTendrilsFolder");
    set_all_platform_paths(&mut given, &[temp_parent_folder]);

    pull_tendril(&given_tendrils_folder, &given).unwrap();

    assert!(given_tendrils_folder.join("SomeApp")
        .join("AnotherTendrilsFolder").join("tendrils.json").exists()
    );
    assert!(given_tendrils_folder.join("SomeApp")
        .read_dir().unwrap().count() == 1
    );
}

#[test]
fn resolved_source_path_is_file_and_dest_is_dir_returns_type_mismatch_error() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
    let source = &temp_parent_folder.join("misc");
    let dest = given_tendrils_folder.join("SomeApp").join("misc");
    write(&source, "Source file contents").unwrap();
    create_dir_all(&dest).unwrap();

    let mut given = Tendril::new("SomeApp", "misc");
    set_all_platform_paths(&mut given, &[temp_parent_folder]);

    let actual = pull_tendril(&given_tendrils_folder, &given).unwrap_err();

    let source_file_contents = read_to_string(source).unwrap();
    assert_eq!(source_file_contents, "Source file contents");
    assert!(matches!(actual, PushPullError::TypeMismatch));
    assert!(dest.is_dir());
    assert!(is_empty(&dest));
}

#[test]
fn resolved_source_path_is_dir_and_dest_is_file_returns_type_mismatch_error() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
    let source = &temp_parent_folder.join("misc");
    let dest = given_tendrils_folder.join("SomeApp").join("misc");
    create_dir_all(&source).unwrap();
    create_dir_all(&dest.parent().unwrap()).unwrap();
    write(&dest, "Dest file contents").unwrap();

    let mut given = Tendril::new("SomeApp", "misc");
    set_all_platform_paths(&mut given, &[temp_parent_folder]);

    let actual = pull_tendril(&given_tendrils_folder, &given).unwrap_err();

    let dest_file_content = read_to_string(dest).unwrap();
    assert_eq!(dest_file_content, "Dest file contents");
    assert!(matches!(actual, PushPullError::TypeMismatch));
    assert!(source.is_dir());
    assert!(is_empty(&source));
}

#[test]
fn resolved_source_path_is_symlink_returns_type_mismatch_error() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = temp_parent_folder.join("TendrilsFolder");
    let original_file = temp_parent_folder.join("original.txt");
    let original_folder = temp_parent_folder.join("original");
    let symlink_file = temp_parent_folder.join("symfile");
    let symlink_folder = temp_parent_folder.join("symdir");
    write(&original_file, "Original file contents").unwrap();
    create_dir_all(&original_folder).unwrap();

    // Create symlinks
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    #[cfg(unix)]
    symlink(original_file, symlink_file).unwrap();
    #[cfg(unix)]
    symlink(original_folder, symlink_folder).unwrap();

    #[cfg(windows)]
    use std::os::windows::fs::{symlink_dir, symlink_file};
    #[cfg(windows)]
    unimplemented!();

    // Note, each of these tendrils could be considered a file tendril or a folder tendril
    // as the type is determined by the type of the source file system object
    let mut given_pointing_to_symfile = Tendril::new("SomeApp", "symfile");
    let mut given_pointing_to_symdir = Tendril::new("SomeApp", "symdir");
    set_all_platform_paths(&mut given_pointing_to_symfile, &[temp_parent_folder.clone()]);
    set_all_platform_paths(&mut given_pointing_to_symdir, &[temp_parent_folder]);

    let actual_1 = pull_tendril(
        &given_tendrils_folder,
        &given_pointing_to_symfile
    ).unwrap_err();
    let actual_2 = pull_tendril(
        &given_tendrils_folder, &given_pointing_to_symdir
    ).unwrap_err();

    assert!(matches!(actual_1, PushPullError::TypeMismatch));
    assert!(matches!(actual_2, PushPullError::TypeMismatch));
    assert!(is_empty(&given_tendrils_folder));
}

#[test]
fn dest_is_symlink_returns_type_mismatch_error() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = temp_parent_folder.join("TendrilsFolder");
    let source_file = temp_parent_folder.join("symfile");
    let source_folder = temp_parent_folder.join("symdir");
    let original_file = given_tendrils_folder.join("SomeApp").join("original.txt");
    let original_folder = given_tendrils_folder
        .join("SomeApp")
        .join("original");
    let symlink_file = given_tendrils_folder
        .join("SomeApp")
        .join("symfile");
    let symlink_folder = given_tendrils_folder
        .join("SomeApp")
        .join("symdir");
    create_dir_all(&original_folder).unwrap();
    create_dir_all(&source_folder).unwrap();
    write(&original_file, "Original file contents").unwrap();
    write(&source_file, "Source file contents").unwrap();

    // Create symlinks
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    #[cfg(unix)]
    symlink(original_file, symlink_file).unwrap();
    #[cfg(unix)]
    symlink(original_folder, symlink_folder).unwrap();

    #[cfg(windows)]
    use std::os::windows::fs::{symlink_dir, symlink_file};
    #[cfg(windows)]
    unimplemented!();

    // Note, each of these tendrils could be considered a file tendril or a folder tendril
    // as the type is determined by the type of the source file system object
    let mut given_pointing_to_symfile = Tendril::new("SomeApp", "symfile");
    let mut given_pointing_to_symdir = Tendril::new("SomeApp", "symdir");
    set_all_platform_paths(&mut given_pointing_to_symfile, &[temp_parent_folder.clone()]);
    set_all_platform_paths(&mut given_pointing_to_symdir, &[temp_parent_folder]);

    let actual_1 = pull_tendril(
        &given_tendrils_folder,
        &given_pointing_to_symfile
    ).unwrap_err();
    let actual_2 = pull_tendril(
        &given_tendrils_folder, &given_pointing_to_symdir
    ).unwrap_err();

    assert!(matches!(actual_1, PushPullError::TypeMismatch));
    assert!(matches!(actual_2, PushPullError::TypeMismatch));
}

#[test]
fn no_read_access_from_source_file_returns_io_error_permission_denied() {
    let temp_tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "TendrilsFolder"
    ).unwrap().into_path();
    // Note: This test sample is not version controlled and must first
    // be created using the setup script - See dev/setup-tendrils.nu
    let source = &temp_tendrils_folder
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .join("tests")
        .join("samples")
        .join("NoReadAccess")
        .join("no_read_access.txt");

    let mut given = Tendril::new("SomeApp", "no_read_access.txt");
    set_all_platform_paths(&mut given, &[source.parent().unwrap().to_owned()]);

    let actual = pull_tendril(&temp_tendrils_folder, &given);

    match actual {
        Err(PushPullError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied)
        },
        _ => panic!()
    }
    assert!(is_empty(&temp_tendrils_folder.join("SomeApp")));
}

#[test]
fn no_read_access_from_source_folder_returns_io_error_permission_denied() {
    let temp_tendrils_folder = TempDir::new_in(
        get_disposable_folder(),
        "TendrilsFolder"
    ).unwrap().into_path();
    // Note: This test sample is not version controlled and must first
    // be created using the setup script - See dev/setup-tendrils.nu
    let source = &temp_tendrils_folder
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .join("tests")
        .join("samples")
        .join("NoReadAccess")
        .join("no_read_access_folder");

    let mut given = Tendril::new("SomeApp", "no_read_access_folder");
    set_all_platform_paths(&mut given, &[source.parent().unwrap().to_owned()]);

    let actual = pull_tendril(&temp_tendrils_folder, &given);

    match actual {
        Err(PushPullError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied)
        },
        _ => panic!()
    }
    assert!(is_empty(&temp_tendrils_folder.join("SomeApp")));
}

#[test]
fn no_write_access_at_dest_file_returns_io_error_permission_denied() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
    let source = &temp_parent_folder.join("misc.txt");
    let dest = &given_tendrils_folder.join("SomeApp").join("misc.txt");
    File::create(&source).unwrap();
    create_dir_all(&dest.parent().unwrap()).unwrap();
    write(&dest, "Don't touch me").unwrap();

    // Set file read-only
    let mut perms = metadata(&dest).unwrap().permissions();
    perms.set_readonly(true);
    set_permissions(&dest, perms).unwrap();

    let mut given = Tendril::new("SomeApp", "misc.txt");
    set_all_platform_paths(&mut given, &[temp_parent_folder]);

    let actual = pull_tendril(&given_tendrils_folder, &given);

    match actual {
        Err(PushPullError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied)
        },
        _ => panic!()
    }
    let dest_file_contents = read_to_string(&dest).unwrap();
    assert_eq!(dest_file_contents, "Don't touch me");
}

#[rstest]
#[case(true)]
#[case(false)]
fn file_tendril_overwrites_dest_file_regardless_of_folder_merge(
    #[case] folder_merge: bool
) {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = temp_parent_folder.join("TendrilsFolder");
    let source = &temp_parent_folder.join("misc.txt");
    let dest = &given_tendrils_folder.join("SomeApp").join("misc.txt");
    write(&source, "Source file contents").unwrap();
    create_dir_all(&dest.parent().unwrap()).unwrap();
    write(&dest, "Overwrite me!").unwrap();

    let mut given = Tendril::new("SomeApp", "misc.txt");
    given.folder_merge = folder_merge;
    set_all_platform_paths(&mut given, &[temp_parent_folder]);

    pull_tendril(&given_tendrils_folder, &given).unwrap();

    let dest_file_contents = read_to_string(&dest).unwrap();
    assert_eq!(dest_file_contents, "Source file contents");
}

#[test]
fn folder_merge_false_w_folder_tendril_overwrites_dest_folder_recursively() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
    let source= &temp_parent_folder.join("SourceFolder");
    let nested_folder= &source.join("NestedFolder");
    let source_misc_file = source.join("misc.txt");
    let source_nested_file = source.join("NestedFolder").join("nested.txt");
    let source_new_nested_file = source.join("NestedFolder").join("new_nested.txt");
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
    set_all_platform_paths(&mut given, &[temp_parent_folder]);

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
    ).unwrap().into_path();
    let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
    let source= &temp_parent_folder.join("SourceFolder");
    let nested_folder= &source.join("NestedFolder");
    let source_misc_file = source.join("misc.txt");
    let source_nested_file = source.join("NestedFolder").join("nested.txt");
    let source_new_nested_file = source.join("NestedFolder").join("new_nested.txt");
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
    set_all_platform_paths(&mut given, &[temp_parent_folder]);

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
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = &temp_parent_folder
        .join("TendrilsFolderThatDoesNotExistYet");
    let source = &temp_parent_folder.join("misc.txt");
    let dest = &given_tendrils_folder
        .join("SomeApp")
        .join("misc.txt");
    write(&source, "Source file contents").unwrap();

    let mut given = Tendril::new("SomeApp", "misc.txt");
    set_all_platform_paths(&mut given, &[temp_parent_folder]);

    pull_tendril(&given_tendrils_folder, &given).unwrap();

    let dest_file_contents = read_to_string(&dest).unwrap();
    assert_eq!(dest_file_contents, "Source file contents");
}

#[test]
fn file_tendril_source_is_unchanged() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
    let source= &temp_parent_folder.join("misc.txt");
    write(source, "Source file contents").unwrap();
    let dest = given_tendrils_folder
        .join("SomeApp")
        .join("misc.txt");

    let mut given = Tendril::new("SomeApp", "misc.txt");
    set_all_platform_paths(&mut given, &[temp_parent_folder]);

    pull_tendril(&given_tendrils_folder, &given).unwrap();

    let dest_contents = read_to_string(dest).unwrap();
    assert_eq!(dest_contents, "Source file contents");

    // Check that source is unchanged
    let source_contents = read_to_string(source).unwrap();
    assert_eq!(source_contents, "Source file contents");
}

#[test]
fn other_tendrils_in_same_app_folder_are_unchanged() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
    let source= &temp_parent_folder.join("misc.txt");
    let some_other_tendril= &given_tendrils_folder.join("SomeApp").join("other.txt");
    create_dir_all(given_tendrils_folder.join("SomeApp")).unwrap();
    write(source, "Source file contents").unwrap();
    write(some_other_tendril, "Another tendril from the same app").unwrap();

    let mut given = Tendril::new("SomeApp", "misc.txt");
    set_all_platform_paths(&mut given, &[temp_parent_folder]);

    pull_tendril(&given_tendrils_folder, &given).unwrap();

    // Check that other tendril is unchanged
    let some_other_tendril_contents = read_to_string(some_other_tendril).unwrap();
    assert_eq!(some_other_tendril_contents, "Another tendril from the same app");
}

#[test]
fn folder_tendril_copies_all_contents_recursively_and_source_is_unchanged() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
    let source= &temp_parent_folder.join("SourceFolder");
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
    set_all_platform_paths(&mut given, &[temp_parent_folder]);

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
    ).unwrap().into_path();
    let given_tendrils_folder = &temp_grandparent_folder.join("TendrilsFolder");
    let parent_mac = temp_grandparent_folder.join("Mac");
    let parent_win = temp_grandparent_folder.join("Windows");
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
    ).unwrap().into_path();
    let given_tendrils_folder = temp_grandparent_folder.join("TendrilsFolder");
    let given_parent_folder_1 = temp_grandparent_folder.join("Parent1");
    let given_parent_folder_2 = temp_grandparent_folder.join("Parent2");
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
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap().into_path();
    let given_tendrils_folder = &temp_parent_folder.join("TendrilsFolder");
    let source = temp_parent_folder.join("misc.txt");
    write(&source, "Source file contents").unwrap();

    let mut tendril = Tendril::new("SomeApp", "misc.txt");
    set_all_platform_paths(
        &mut tendril,
        &[PathBuf::from("I_Do_Not_Exist"), source]
    );

    let actual = pull_tendril(&given_tendrils_folder, &tendril);

    match actual {
        Err(PushPullError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound)
        },
        _ => panic!(),
    }
    assert!(is_empty(given_tendrils_folder));
}
