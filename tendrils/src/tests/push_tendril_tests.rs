//! Contains tests specific to push actions.
//! See also [`crate::tests::common_action_tests`].

use crate::test_utils::{
    set_ra,
    symlink_expose,
    Setup,
};
use crate::{
    push_tendril,
    ActionLog,
    FsoType,
    Location,
    Tendril,
    TendrilActionError,
    TendrilActionSuccess,
    TendrilMode,
};
use rstest::rstest;
use rstest_reuse::{self, apply};
use std::fs::{
    create_dir_all,
    metadata,
    read_to_string,
    set_permissions,
    write,
};

/// See also [`crate::tests::common_action_tests::remote_is_unchanged`] for
/// `dry_run` case
#[apply(crate::tendril::tests::tendril_tests::valid_groups_and_names)]
fn remote_parent_and_local_exist_copies_to_remote(
    #[case] name: &str,
    #[values(true, false)] force: bool,
    #[values(true, false)] as_dir: bool,
) {
    let mut setup = Setup::new();
    setup.remote_file = setup.parent_dir.join(&name);
    setup.remote_dir = setup.parent_dir.join(&name);
    setup.remote_nested_file = setup.remote_dir.join("nested.txt");
    setup.local_file = setup.group_dir.join(&name);
    setup.local_dir = setup.group_dir.join(&name);
    setup.local_nested_file = setup.local_dir.join("nested.txt");
    let exp_local_type;
    if as_dir {
        setup.make_local_nested_file();
        exp_local_type = Some(FsoType::Dir);
    }
    else {
        setup.make_local_file();
        exp_local_type = Some(FsoType::File);
    }

    let tendril = Tendril::new_expose(
        "SomeApp",
        name,
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();

    let actual = push_tendril(&setup.td_repo, &tendril, false, force);

    assert_eq!(
        actual,
        ActionLog::new(
            exp_local_type,
            None,
            setup.remote_file.clone(),
            Ok(TendrilActionSuccess::New),
        )
    );
    if as_dir {
        assert_eq!(
            setup.remote_nested_file_contents(),
            "Local nested file contents"
        );
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
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_target_file();
    setup.make_target_nested_file();
    setup.make_group_dir();
    symlink_expose(&setup.local_file, &setup.target_file, false, false)
        .unwrap();
    symlink_expose(&setup.local_dir, &setup.target_dir, false, false).unwrap();

    let file_tendril = setup.file_tendril();
    let dir_tendril = setup.dir_tendril();

    let file_actual =
        push_tendril(&setup.td_repo, &file_tendril, dry_run, force);
    let dir_actual = push_tendril(&setup.td_repo, &dir_tendril, dry_run, force);

    let exp_file_result;
    let exp_dir_result;
    match (dry_run, force) {
        (_, false) => {
            exp_file_result = Err(TendrilActionError::TypeMismatch {
                loc: Location::Source,
                mistype: FsoType::SymFile,
            });
            exp_dir_result = Err(TendrilActionError::TypeMismatch {
                loc: Location::Source,
                mistype: FsoType::SymDir,
            });
        }
        (false, true) => {
            exp_file_result = Ok(TendrilActionSuccess::New);
            exp_dir_result = Ok(TendrilActionSuccess::New);
        }
        (true, true) => {
            exp_file_result = Ok(TendrilActionSuccess::NewSkipped);
            exp_dir_result = Ok(TendrilActionSuccess::NewSkipped);
        }
    };

    assert_eq!(
        file_actual,
        ActionLog::new(
            Some(FsoType::SymFile),
            None,
            setup.remote_file.clone(),
            exp_file_result,
        )
    );
    assert_eq!(
        dir_actual,
        ActionLog::new(
            Some(FsoType::SymDir),
            None,
            setup.remote_dir.clone(),
            exp_dir_result,
        )
    );
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
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_nested_file();
    setup.make_target_file();
    setup.make_target_nested_file();
    symlink_expose(&setup.remote_file, &setup.target_file, false, true)
        .unwrap();
    symlink_expose(&setup.remote_dir, &setup.target_dir, false, true).unwrap();

    let file_tendril = setup.file_tendril();
    let dir_tendril = setup.dir_tendril();

    let file_actual =
        push_tendril(&setup.td_repo, &file_tendril, dry_run, force);
    let dir_actual = push_tendril(&setup.td_repo, &dir_tendril, dry_run, force);

    let exp_file_result;
    let exp_dir_result;
    match (dry_run, force) {
        (_, false) => {
            exp_file_result = Err(TendrilActionError::TypeMismatch {
                loc: Location::Dest,
                mistype: FsoType::SymFile,
            });
            exp_dir_result = Err(TendrilActionError::TypeMismatch {
                loc: Location::Dest,
                mistype: FsoType::SymDir,
            });
        }
        (false, true) => {
            exp_file_result = Ok(TendrilActionSuccess::Overwrite);
            exp_dir_result = Ok(TendrilActionSuccess::Overwrite);
        }
        (true, true) => {
            exp_file_result = Ok(TendrilActionSuccess::OverwriteSkipped);
            exp_dir_result = Ok(TendrilActionSuccess::OverwriteSkipped);
        }
    };

    assert_eq!(
        file_actual,
        ActionLog::new(
            Some(FsoType::File),
            Some(FsoType::SymFile),
            setup.remote_file.clone(),
            exp_file_result,
        )
    );
    assert_eq!(
        dir_actual,
        ActionLog::new(
            Some(FsoType::Dir),
            Some(FsoType::SymDir),
            setup.remote_dir.clone(),
            exp_dir_result,
        )
    );
    if force && !dry_run {
        assert!(!setup.remote_file.is_symlink());
        assert!(!setup.remote_dir.is_symlink());
        assert_eq!(setup.remote_file_contents(), "Local file contents");
        assert_eq!(
            setup.remote_nested_file_contents(),
            "Local nested file contents"
        );
    }
    else {
        assert!(setup.remote_file.is_symlink());
        assert!(setup.remote_dir.is_symlink());
        assert_eq!(setup.remote_file_contents(), "Target file contents");
        assert_eq!(
            setup.remote_nested_file_contents(),
            "Target nested file contents"
        );
    }
}

// AKA `source_is_file_and_dest_is_dir`
#[rstest]
#[case(TendrilMode::DirMerge)]
#[case(TendrilMode::DirOverwrite)]
fn local_is_file_and_remote_is_dir_returns_type_mismatch_error_unless_forced(
    #[case] mode: TendrilMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    create_dir_all(&setup.remote_file).unwrap();

    let mut tendril = setup.file_tendril();
    tendril.mode = mode;

    let actual = push_tendril(&setup.td_repo, &tendril, dry_run, force);

    let exp_result = match (dry_run, force) {
        (_, false) => Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: FsoType::Dir,
        }),
        (false, true) => Ok(TendrilActionSuccess::Overwrite),
        (true, true) => Ok(TendrilActionSuccess::OverwriteSkipped),
    };

    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::File),
            Some(FsoType::Dir),
            setup.remote_file.clone(),
            exp_result,
        )
    );
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
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_local_nested_file();
    write(&setup.remote_dir, "I'm a file!").unwrap();

    let mut tendril = setup.dir_tendril();
    tendril.mode = mode;

    let actual = push_tendril(&setup.td_repo, &tendril, dry_run, force);

    let exp_result = match (dry_run, force) {
        (_, false) => Err(TendrilActionError::TypeMismatch {
            loc: Location::Dest,
            mistype: FsoType::File,
        }),
        (false, true) => Ok(TendrilActionSuccess::Overwrite),
        (true, true) => Ok(TendrilActionSuccess::OverwriteSkipped),
    };

    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::Dir),
            Some(FsoType::File),
            setup.remote_dir.clone(),
            exp_result,
        )
    );
    assert!(setup.local_dir.is_dir());
    if force && !dry_run {
        assert_eq!(
            setup.remote_nested_file_contents(),
            "Local nested file contents"
        );
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
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_local_file();

    let mut tendril = setup.file_tendril();
    tendril.mode = mode;

    let actual = push_tendril(&setup.td_repo, &tendril, false, force);

    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::File),
            Some(FsoType::File),
            setup.remote_file.clone(),
            Ok(TendrilActionSuccess::Overwrite),
        )
    );
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

    let actual = push_tendril(&setup.td_repo, &tendril, false, force);

    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::Dir),
            Some(FsoType::Dir),
            setup.remote_dir.clone(),
            Ok(TendrilActionSuccess::Overwrite),
        )
    );
    let remote_new_2nested_file_contents =
        read_to_string(remote_new_2nested_file).unwrap();
    assert_eq!(
        setup.remote_nested_file_contents(),
        "Local nested file contents"
    );
    assert_eq!(remote_new_2nested_file_contents, "I'm not in the remote dir");
    assert!(!remote_extra_2nested_file.exists());
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_merge_w_dir_tendril_merges_w_local_dir_recursively(#[case] force: bool) {
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

    let actual = push_tendril(&setup.td_repo, &tendril, false, force);

    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::Dir),
            Some(FsoType::Dir),
            setup.remote_dir.clone(),
            Ok(TendrilActionSuccess::Overwrite),
        )
    );
    let remote_new_2nested_file_contents =
        read_to_string(remote_new_2nested_file).unwrap();
    let remote_extra_2nested_file_contents =
        read_to_string(remote_extra_2nested_file).unwrap();
    assert_eq!(
        setup.remote_nested_file_contents(),
        "Local nested file contents"
    );
    assert_eq!(remote_new_2nested_file_contents, "I'm not in the remote dir");
    assert_eq!(remote_extra_2nested_file_contents, "I'm not in the local dir");
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_overwrite_w_subdir_dir_tendril_replaces_remote_dir_recursively(
    #[case] force: bool,
) {
    let setup = Setup::new();
    let remote_nested_dir = &setup.remote_subdir_dir.join("NestedDir");
    let remote_new_2nested_file = remote_nested_dir.join("new_nested.txt");
    let remote_extra_2nested_file = remote_nested_dir.join("extra_nested.txt");
    let local_nested_dir = &setup.local_subdir_dir.join("NestedDir");
    let local_new_2nested_file = local_nested_dir.join("new_nested.txt");
    setup.make_remote_subdir_nested_file();
    setup.make_local_subdir_nested_file();
    create_dir_all(&remote_nested_dir).unwrap();
    create_dir_all(&local_nested_dir).unwrap();
    write(&local_new_2nested_file, "I'm not in the remote dir").unwrap();
    write(&remote_extra_2nested_file, "I'm not in the local dir").unwrap();

    let mut tendril = setup.subdir_dir_tendril();
    tendril.mode = TendrilMode::DirOverwrite;

    let actual = push_tendril(&setup.td_repo, &tendril, false, force);

    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::Dir),
            Some(FsoType::Dir),
            setup.remote_subdir_dir.clone(),
            Ok(TendrilActionSuccess::Overwrite),
        )
    );
    let remote_new_2nested_file_contents =
        read_to_string(remote_new_2nested_file).unwrap();
    assert_eq!(
        setup.remote_subdir_nested_file_contents(),
        "Local subdir nested file contents"
    );
    assert_eq!(remote_new_2nested_file_contents, "I'm not in the remote dir");
    assert!(!remote_extra_2nested_file.exists());
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_merge_w_subdir_dir_tendril_merges_w_local_dir_recursively(
    #[case] force: bool,
) {
    let setup = Setup::new();
    let remote_nested_dir = &setup.remote_subdir_dir.join("NestedDir");
    let remote_new_2nested_file = remote_nested_dir.join("new_nested.txt");
    let remote_extra_2nested_file = remote_nested_dir.join("extra_nested.txt");
    let local_nested_dir = &setup.local_subdir_dir.join("NestedDir");
    let local_new_2nested_file = local_nested_dir.join("new_nested.txt");
    setup.make_remote_subdir_nested_file();
    setup.make_local_subdir_nested_file();
    create_dir_all(&remote_nested_dir).unwrap();
    create_dir_all(&local_nested_dir).unwrap();
    write(&local_new_2nested_file, "I'm not in the remote dir").unwrap();
    write(&remote_extra_2nested_file, "I'm not in the local dir").unwrap();

    let mut tendril = setup.subdir_dir_tendril();
    tendril.mode = TendrilMode::DirMerge;

    let actual = push_tendril(&setup.td_repo, &tendril, false, force);

    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::Dir),
            Some(FsoType::Dir),
            setup.remote_subdir_dir.clone(),
            Ok(TendrilActionSuccess::Overwrite),
        )
    );
    let remote_new_2nested_file_contents =
        read_to_string(remote_new_2nested_file).unwrap();
    let remote_extra_2nested_file_contents =
        read_to_string(remote_extra_2nested_file).unwrap();
    assert_eq!(
        setup.remote_subdir_nested_file_contents(),
        "Local subdir nested file contents"
    );
    assert_eq!(remote_new_2nested_file_contents, "I'm not in the remote dir");
    assert_eq!(remote_extra_2nested_file_contents, "I'm not in the local dir");
}

#[rstest]
#[case(true)]
#[case(false)]
fn no_read_access_from_local_file_returns_io_error_permission_denied_unless_dry_run(
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_td_repo_dir();
    setup.make_local_nra_file();

    let tendril = Tendril::new_expose(
        "SomeApp",
        "nra.txt",
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();

    let actual = push_tendril(&setup.td_repo, &tendril, dry_run, force);

    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::NewSkipped);
    }
    else {
        exp_result = Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::PermissionDenied,
            loc: Location::Source,
        });
    }
    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::File),
            None,
            setup.remote_nra_file.clone(),
            exp_result,
        )
    );
    assert!(!setup.remote_nra_file.exists());
}

#[rstest]
#[case(true)]
#[case(false)]
fn no_read_access_from_local_dir_returns_io_error_permission_denied_unless_dry_run(
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_td_repo_dir();
    setup.make_local_nra_dir();

    let tendril = Tendril::new_expose(
        "SomeApp",
        "nra",
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    )
    .unwrap();

    let actual = push_tendril(&setup.td_repo, &tendril, dry_run, force);

    set_ra(&setup.local_nra_dir, true);
    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::NewSkipped);
    }
    else {
        exp_result = Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::PermissionDenied,
            loc: Location::Source,
        });
    }
    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::Dir),
            None,
            setup.remote_nra_dir.clone(),
            exp_result,
        )
    );
    assert!(!setup.remote_nra_dir.exists());
}

#[rstest]
#[case(true)]
#[case(false)]
fn no_write_access_at_remote_file_returns_io_error_permission_denied_unless_dry_run(
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_local_file();

    // Set file read-only
    let mut perms = metadata(&setup.remote_file).unwrap().permissions();
    perms.set_readonly(true);
    set_permissions(&setup.remote_file, perms).unwrap();
    if std::env::consts::FAMILY == "unix" {
        // Unix does not consider permissions on the file when deleting, only on
        // its parent
        let mut parent_perms =
            metadata(&setup.parent_dir).unwrap().permissions();
        parent_perms.set_readonly(true);
        set_permissions(&setup.parent_dir, parent_perms).unwrap();
    }

    let tendril = setup.file_tendril();

    let actual = push_tendril(&setup.td_repo, &tendril, dry_run, force);

    // Cleanup
    let mut parent_perms = metadata(&setup.parent_dir).unwrap().permissions();
    parent_perms.set_readonly(false);
    set_permissions(&setup.parent_dir, parent_perms).unwrap();

    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_result = Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::PermissionDenied,
            loc: Location::Dest,
        });
    }
    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::File),
            Some(FsoType::File),
            setup.remote_file.clone(),
            exp_result,
        )
    );
    assert_eq!(setup.remote_file_contents(), "Remote file contents");
}

#[rstest]
#[case(true)]
#[case(false)]
#[cfg_attr(windows, ignore)] // These permissions do not prevent write access on
                             // Windows. This must be done through the Security
                             // interface
fn no_write_access_at_remote_dir_returns_io_error_permission_denied_unless_dry_run(
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_nested_file();
    setup.make_local_nested_file();

    // Set dir read-only
    let mut perms = metadata(&setup.remote_dir).unwrap().permissions();
    perms.set_readonly(true);
    set_permissions(&setup.remote_dir, perms.clone()).unwrap();

    let tendril = setup.dir_tendril();

    let actual = push_tendril(&setup.td_repo, &tendril, dry_run, force);

    // Cleanup
    perms.set_readonly(false);
    set_permissions(&setup.remote_dir, perms).unwrap();

    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::OverwriteSkipped);
    }
    else {
        exp_result = Err(TendrilActionError::IoError {
            kind: std::io::ErrorKind::PermissionDenied,
            loc: Location::Dest,
        });
    }
    assert_eq!(
        actual,
        ActionLog::new(
            Some(FsoType::Dir),
            Some(FsoType::Dir),
            setup.remote_dir.clone(),
            exp_result,
        )
    );
    assert_eq!(
        setup.remote_nested_file_contents(),
        "Remote nested file contents"
    );
}
