//! Contains tests specific to push actions.
//! See also [`crate::tests::common_action_tests`].

use crate::{
    push_tendril,
    symlink,
    TendrilActionError,
    TendrilActionSuccess,
    TendrilMode
};
use crate::test_utils::{get_disposable_dir, get_samples_dir, is_empty, Setup};
use crate::tendril::Tendril;
use rstest::rstest;
use rstest_reuse::{self, apply};
use std::fs::{create_dir_all, metadata, read_to_string, set_permissions, write};
use tempdir::TempDir;

/// See also [`crate::tests::common_action_tests::remote_is_unchanged`] for
/// `dry_run` case
#[apply(crate::tests::tendril_tests::valid_groups_and_names)]
fn remote_parent_and_local_exist_copies_to_remote(
    #[case] name: &str,

    #[values(true, false)]
    force: bool,

    #[values(true, false)]
    as_dir: bool,
) {
    let mut setup = Setup::new();
    setup.remote_file = setup.parent_dir.join(&name);
    setup.remote_dir = setup.parent_dir.join(&name);
    setup.remote_nested_file = setup.remote_dir.join("nested.txt");
    setup.local_file = setup.group_dir.join(&name);
    setup.local_dir = setup.group_dir.join(&name);
    setup.local_nested_file = setup.local_dir.join("nested.txt");
    if as_dir {
        setup.make_local_nested_file();
    }
    else {
        setup.make_local_file();
    }

    let tendril = Tendril::new(
        "SomeApp",
        name,
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = push_tendril(&setup.td_dir, &tendril, false, force);

    assert_eq!(actual, Ok(TendrilActionSuccess::New));
    if as_dir {
        assert_eq!(setup.remote_nested_file_contents(), "Local nested file contents");
    }
    else {
        assert_eq!(setup.remote_file_contents(), "Local file contents");
    }
    assert_eq!(setup.group_dir.read_dir().iter().count(), 1);
}

#[rstest]
#[case(true)]
#[case(false)]
fn local_is_symlink_returns_type_mismatch_error_unless_forced_then_copies_symlink_target_contents_keeps_local_name(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_target_file();
    setup.make_target_nested_file();
    setup.make_group_dir();
    symlink(&setup.local_file, &setup.target_file, false, false).unwrap();
    symlink(&setup.local_dir, &setup.target_dir, false, false).unwrap();

    let file_tendril = setup.file_tendril();
    let dir_tendril = setup.dir_tendril();

    let file_actual = push_tendril(
        &setup.td_dir,
        &file_tendril,
        dry_run,
        force,
    );
    let dir_actual = push_tendril(
        &setup.td_dir,
        &dir_tendril,
        dry_run,
        force,
    );

    match (dry_run, force) {
        (_, false) => {
            assert_eq!(file_actual, Err(TendrilActionError::TypeMismatch));
            assert_eq!(dir_actual, Err(TendrilActionError::TypeMismatch));
        },
        (false, true) => {
            assert_eq!(file_actual, Ok(TendrilActionSuccess::New));
            assert_eq!(dir_actual, Ok(TendrilActionSuccess::New));
        },
        (true, true) => {
            assert_eq!(file_actual, Ok(TendrilActionSuccess::NewSkipped));
            assert_eq!(dir_actual, Ok(TendrilActionSuccess::NewSkipped));
        },
    }

    assert!(setup.local_file.is_symlink());
    assert!(setup.local_dir.is_symlink());
    assert!(!setup.remote_file.is_symlink());
    assert!(!setup.remote_dir.is_symlink());
    if force && !dry_run {
        assert_eq!(setup.remote_file.file_name().unwrap(), "misc.txt");
        assert_eq!(setup.remote_file_contents(), "Target file contents");
        assert_eq!(setup.remote_dir.file_name().unwrap(), "misc");
        assert_eq!(
            setup.remote_nested_file_contents(),
            "Target nested file contents"
        );
    }
    else {
        assert!(!setup.remote_file.exists());
        assert!(!setup.remote_dir.exists());
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn remote_is_symlink_returns_type_mismatch_error_unless_forced(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_nested_file();
    setup.make_target_file();
    setup.make_target_nested_file();
    symlink(&setup.remote_file, &setup.target_file, false, true).unwrap();
    symlink(&setup.remote_dir, &setup.target_dir, false, true).unwrap();

    let file_tendril = setup.file_tendril();
    let dir_tendril = setup.dir_tendril();

    let file_actual = push_tendril(
        &setup.td_dir,
        &file_tendril,
        dry_run,
        force,
    );
    let dir_actual = push_tendril(
        &setup.td_dir,
        &dir_tendril,
        dry_run,
        force,
    );

    match (dry_run, force) {
        (_, false) => {
            assert_eq!(file_actual, Err(TendrilActionError::TypeMismatch));
            assert_eq!(dir_actual, Err(TendrilActionError::TypeMismatch));
        },
        (false, true) => {
            assert_eq!(file_actual, Ok(TendrilActionSuccess::Overwrite));
            assert_eq!(dir_actual, Ok(TendrilActionSuccess::Overwrite));
        },
        (true, true) => {
            assert_eq!(file_actual, Ok(TendrilActionSuccess::OverwriteSkipped));
            assert_eq!(dir_actual, Ok(TendrilActionSuccess::OverwriteSkipped));
        },
    }

    if force && !dry_run {
        assert!(!setup.remote_file.is_symlink());
        assert!(!setup.remote_dir.is_symlink());
        assert_eq!(setup.remote_file_contents(), "Local file contents");
        assert_eq!(setup.remote_nested_file_contents(), "Local nested file contents");
    }
    else {
        assert!(setup.remote_file.is_symlink());
        assert!(setup.remote_dir.is_symlink());
        assert_eq!(setup.remote_file_contents(), "Target file contents");
        assert_eq!(setup.remote_nested_file_contents(), "Target nested file contents");
    }
}

// AKA `source_is_file_and_dest_is_dir`
#[rstest]
#[case(TendrilMode::DirMerge)]
#[case(TendrilMode::DirOverwrite)]
fn local_is_file_and_remote_is_dir_returns_type_mismatch_error_unless_forced(
    #[case] mode: TendrilMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    create_dir_all(&setup.remote_file).unwrap();

    let mut tendril = setup.file_tendril();
    tendril.mode = mode;

    let actual = push_tendril(&setup.td_dir, &tendril, dry_run, force);

    match (dry_run, force) {
        (_, false) => {
            assert_eq!(actual, Err(TendrilActionError::TypeMismatch));
        },
        (false, true) => {
            assert_eq!(actual, Ok(TendrilActionSuccess::Overwrite));
        },
        (true, true) => {
            assert_eq!(actual, Ok(TendrilActionSuccess::OverwriteSkipped));
        },
    }

    if force && !dry_run {
        assert_eq!(&setup.remote_file_contents(), "Local file contents");
        assert_eq!(&setup.local_file_contents(), "Local file contents");
    }
    else {
        assert!(&setup.remote_file.is_dir());
        assert_eq!(&setup.local_file_contents(), "Local file contents");
    }
}

// AKA `source_is_dir_and_dest_is_file`
#[rstest]
#[case(TendrilMode::DirMerge)]
#[case(TendrilMode::DirOverwrite)]
fn local_is_dir_and_remote_is_file_returns_type_mismatch_error_unless_forced(
    #[case] mode: TendrilMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_local_nested_file();
    write(&setup.remote_dir, "I'm a file!").unwrap();

    let mut tendril = setup.dir_tendril();
    tendril.mode = mode;

    let actual = push_tendril(&setup.td_dir, &tendril, dry_run, force);

    match (dry_run, force) {
        (_, false) => {
            assert_eq!(actual, Err(TendrilActionError::TypeMismatch));
        },
        (false, true) => {
            assert_eq!(actual, Ok(TendrilActionSuccess::Overwrite));
        },
        (true, true) => {
            assert_eq!(actual, Ok(TendrilActionSuccess::OverwriteSkipped));
        },
    }

    assert!(setup.local_dir.is_dir());
    if force && !dry_run {
        assert_eq!(setup.remote_nested_file_contents(), "Local nested file contents");
        assert_eq!(setup.remote_dir.read_dir().iter().count(), 1);
        assert_eq!(setup.local_dir.read_dir().iter().count(), 1);
    }
    else {
        let remote_dir_contents = read_to_string(&setup.remote_dir).unwrap();
        assert_eq!(remote_dir_contents, "I'm a file!");
        assert_eq!(setup.local_dir.read_dir().iter().count(), 1);
    }
}

#[rstest]
#[case(TendrilMode::DirMerge)]
#[case(TendrilMode::DirOverwrite)]
fn file_tendril_overwrites_remote_file_regardless_of_dir_merge_mode(
    #[case] mode: TendrilMode,
    
    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_local_file();

    let mut tendril = setup.file_tendril();
    tendril.mode = mode;

    let actual = push_tendril(&setup.td_dir, &tendril, false, force);

    assert_eq!(actual, Ok(TendrilActionSuccess::Overwrite));
    assert_eq!(setup.remote_file_contents(), "Local file contents");
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_overwrite_w_dir_tendril_replaces_remote_dir_recursively(
    #[case] force: bool,
) {
    let setup = Setup::new();
    let remote_nested_dir = &setup.remote_dir.join("NestedDir");
    let remote_new_2nested_file = remote_nested_dir.join("new_nested.txt");
    let remote_extra_2nested_file = remote_nested_dir.join("extra_nested.txt");
    let local_nested_dir = &setup.local_dir.join("NestedDir");
    let local_new_2nested_file = local_nested_dir.join("new_nested.txt");
    setup.make_remote_nested_file();
    setup.make_local_nested_file();
    create_dir_all(&remote_nested_dir).unwrap();
    create_dir_all(&local_nested_dir).unwrap();
    write(&local_new_2nested_file, "I'm not in the remote dir").unwrap();
    write(&remote_extra_2nested_file, "I'm not in the local dir").unwrap();

    let mut tendril = setup.dir_tendril();
    tendril.mode = TendrilMode::DirOverwrite;

    let actual = push_tendril(&setup.td_dir, &tendril, false, force);

    assert_eq!(actual, Ok(TendrilActionSuccess::Overwrite));
    let remote_new_2nested_file_contents =
        read_to_string(remote_new_2nested_file).unwrap();
    assert_eq!(setup.remote_nested_file_contents(), "Local nested file contents");
    assert_eq!(remote_new_2nested_file_contents, "I'm not in the remote dir");
    assert!(!remote_extra_2nested_file.exists());
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_merge_w_dir_tendril_merges_w_local_dir_recursively(
    #[case] force: bool,
) {
    let setup = Setup::new();
    let remote_nested_dir = &setup.remote_dir.join("NestedDir");
    let remote_new_2nested_file = remote_nested_dir.join("new_nested.txt");
    let remote_extra_2nested_file = remote_nested_dir.join("extra_nested.txt");
    let local_nested_dir = &setup.local_dir.join("NestedDir");
    let local_new_2nested_file = local_nested_dir.join("new_nested.txt");
    setup.make_remote_nested_file();
    setup.make_local_nested_file();
    create_dir_all(&remote_nested_dir).unwrap();
    create_dir_all(&local_nested_dir).unwrap();
    write(&local_new_2nested_file, "I'm not in the remote dir").unwrap();
    write(&remote_extra_2nested_file, "I'm not in the local dir").unwrap();

    let mut tendril = setup.dir_tendril();
    tendril.mode = TendrilMode::DirMerge;

    let actual = push_tendril(&setup.td_dir, &tendril, false, force);

    assert_eq!(actual, Ok(TendrilActionSuccess::Overwrite));
    let remote_new_2nested_file_contents =
        read_to_string(remote_new_2nested_file).unwrap();
    let remote_extra_2nested_file_contents =
        read_to_string(remote_extra_2nested_file).unwrap();
    assert_eq!(setup.remote_nested_file_contents(), "Local nested file contents");
    assert_eq!(remote_new_2nested_file_contents, "I'm not in the remote dir");
    assert_eq!(remote_extra_2nested_file_contents, "I'm not in the local dir");
}

#[rstest]
#[case(true)]
#[case(false)]
fn no_read_access_from_local_file_returns_io_error_permission_denied_unless_dry_run(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();

    // Note: This test sample is not version controlled and must first
    // be created using the setup script - See dev/setup-tendrils.nu
    let td_dir = get_samples_dir();

    let tendril = Tendril::new(
        "NoReadAccess",
        "no_read_access.txt",
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = push_tendril(
        &td_dir,
        &tendril,
        dry_run,
        force,
    );

    assert!(is_empty(temp_parent_dir.path()));
    if dry_run {
        assert_eq!(actual, Ok(TendrilActionSuccess::NewSkipped));
    }
    else {
        assert_eq!(actual, Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::PermissionDenied,
        }));
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn no_read_access_from_local_dir_returns_io_error_permission_denied_unless_dry_run(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();

    // Note: This test sample is not version controlled and must first
    // be created using the setup script - See dev/setup-tendrils.nu
    let td_dir = get_samples_dir();

    let tendril = Tendril::new(
        "NoReadAccess",
        "no_read_access_dir",
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = push_tendril(
        &td_dir,
        &tendril,
        dry_run,
        force,
    );

    assert!(is_empty(temp_parent_dir.path()));
    if dry_run {
        assert_eq!(actual, Ok(TendrilActionSuccess::NewSkipped));
    }
    else {
        assert_eq!(actual, Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::PermissionDenied,
        }));
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn no_write_access_at_remote_file_returns_io_error_permission_denied_unless_dry_run(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_local_file();

    // Set file read-only
    let mut perms = metadata(&setup.remote_file).unwrap().permissions();
    perms.set_readonly(true);
    set_permissions(&setup.remote_file, perms).unwrap();

    let tendril = setup.file_tendril();

    let actual = push_tendril(&setup.td_dir, &tendril, dry_run, force);

    assert_eq!(setup.remote_file_contents(), "Remote file contents");
    if dry_run {
        assert_eq!(actual, Ok(TendrilActionSuccess::OverwriteSkipped));
    }
    else {
        assert_eq!(actual, Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::PermissionDenied,
        }));
    }
}

#[rstest]
#[case(true)]
#[case(false)]
#[cfg_attr(windows, ignore)]
fn no_write_access_at_remote_dir_returns_io_error_permission_denied_unless_dry_run(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_nested_file();
    setup.make_local_nested_file();

    // Set dir read-only
    let mut perms = metadata(&setup.remote_dir).unwrap().permissions();
    perms.set_readonly(true);
    set_permissions(&setup.remote_dir, perms.clone()).unwrap();

    let tendril = setup.dir_tendril();

    let actual = push_tendril(&setup.td_dir, &tendril, dry_run, force);

    // Cleanup
    perms.set_readonly(false);
    set_permissions(&setup.remote_dir, perms).unwrap();

    assert_eq!(setup.remote_nested_file_contents(), "Remote nested file contents");
    if dry_run {
        assert_eq!(actual, Ok(TendrilActionSuccess::OverwriteSkipped));
    }
    else {
        assert_eq!(actual, Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::PermissionDenied,
        }));
    }
}
