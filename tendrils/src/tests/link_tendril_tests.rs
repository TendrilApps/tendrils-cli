//! Contains tests specific to link actions.
//! See also [`crate::tests::common_action_tests`].

use crate::{link_tendril, symlink};
use crate::enums::{TendrilActionError, TendrilActionSuccess};
use crate::resolved_tendril::{ResolvedTendril, TendrilMode};
use crate::test_utils::{
    get_disposable_dir,
    get_samples_dir,
    Setup,
    symlink_expose
};
use rstest::rstest;
use rstest_reuse::{self, apply};
use std::fs::{create_dir_all, metadata, set_permissions};
use std::path::PathBuf;
use tempdir::TempDir;

/// See also [`crate::tests::common_action_tests::remote_is_unchanged`] for
/// `dry_run` case
#[apply(crate::tests::resolved_tendril_tests::valid_groups_and_names)]
fn remote_parent_and_local_exist_symlink_to_local_is_created(
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
    assert!(!setup.remote_file.exists());
    assert!(!setup.remote_dir.exists());

    let tendril =  ResolvedTendril::new(
        "SomeApp",
        name,
        setup.parent_dir.clone(),
        TendrilMode::Link,
    ).unwrap();

    link_tendril(&setup.td_dir, &tendril, false, force).unwrap();

    use std::env::consts::FAMILY;
    let expected_target: PathBuf;
    if as_dir {
        assert_eq!(
            setup.remote_nested_file_contents(),
            "Local nested file contents"
        );
        assert!(setup.remote_dir.is_symlink());
        if FAMILY == "windows" && name.ends_with('.') {
            // Trailing dots are dropped by the Windows filesystem
            let stripped_name = &name[..name.len()-1];
            expected_target = setup.local_dir
                .parent()
                .unwrap()
                .join(stripped_name);
        }
        else {
            expected_target = setup.local_dir;
        }
        assert_eq!(
            std::fs::read_link(setup.remote_dir).unwrap(),
            expected_target,
        );
    }
    else {
        assert_eq!(setup.remote_file_contents(), "Local file contents");
        assert!(setup.remote_file.is_symlink());
        if FAMILY == "windows" && name.ends_with('.') {
            // Trailing dots are dropped by the Windows filesystem
            let stripped_name = &name[..name.len()-1];
            expected_target = setup.local_file
                .parent()
                .unwrap()
                .join(stripped_name);
        }
        else {
            expected_target = setup.local_file;
        }
        assert_eq!(
            std::fs::read_link(setup.remote_file).unwrap(),
            expected_target
        );
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn remote_exists_and_is_not_symlink_returns_type_mismatch_error_unless_forced(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_remote_nested_file();
    setup.make_local_file();
    setup.make_local_nested_file();

    let mut file_tendril = setup.resolved_file_tendril();
    file_tendril.mode = TendrilMode::Link;

    let mut dir_tendril = setup.resolved_dir_tendril();
    dir_tendril.mode = TendrilMode::Link;
    
    let file_actual = link_tendril(
        &setup.td_dir,
        &file_tendril,
        dry_run,
        force,
    );
    let dir_actual = link_tendril(
        &setup.td_dir,
        &dir_tendril,
        dry_run,
        force,
    );

    match (dry_run, force) {
        (_, false) => {
            assert!(matches!(file_actual, Err(TendrilActionError::TypeMismatch)));
            assert!(matches!(dir_actual, Err(TendrilActionError::TypeMismatch)));
        },
        (false, true) => {
            assert!(matches!(file_actual, Ok(TendrilActionSuccess::Ok)));
            assert!(matches!(dir_actual, Ok(TendrilActionSuccess::Ok)));
        },
        (true, true) => {
            assert!(matches!(file_actual, Ok(TendrilActionSuccess::Skipped)));
            assert!(matches!(dir_actual, Ok(TendrilActionSuccess::Skipped)));
        },
    }

    if force && !dry_run {
        assert!(setup.remote_file.is_symlink());
        assert!(setup.remote_dir.is_symlink());
        assert_eq!(setup.remote_file_contents(), "Local file contents");
        assert_eq!(
            setup.remote_nested_file_contents(),
            "Local nested file contents"
        );
    }
    else {
        assert!(!setup.remote_file.is_symlink());
        assert!(!setup.remote_dir.is_symlink());
        assert_eq!(setup.remote_file_contents(), "Remote file contents");
        assert_eq!(
            setup.remote_nested_file_contents(),
            "Remote nested file contents"
        );
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn local_is_symlink_returns_type_mismatch_error_unless_forced(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_target_file();
    setup.make_target_nested_file();
    create_dir_all(&setup.group_dir).unwrap();
    symlink(&setup.local_file, &setup.target_file, false, false).unwrap();
    symlink(&setup.local_dir, &setup.target_dir, false, false).unwrap();

    let mut file_tendril = setup.resolved_file_tendril();
    file_tendril.mode = TendrilMode::Link;

    let mut dir_tendril = setup.resolved_dir_tendril();
    dir_tendril.mode = TendrilMode::Link;

    let file_actual = link_tendril(
        &setup.td_dir,
        &file_tendril,
        dry_run,
        force,
    );
    let dir_actual = link_tendril(
        &setup.td_dir,
        &dir_tendril,
        dry_run,
        force,
    );

    match (dry_run, force) {
        (_, false) => {
            assert!(matches!(file_actual, Err(TendrilActionError::TypeMismatch)));
            assert!(matches!(dir_actual, Err(TendrilActionError::TypeMismatch)));
        },
        (false, true) => {
            assert!(matches!(file_actual, Ok(TendrilActionSuccess::Ok)));
            assert!(matches!(dir_actual, Ok(TendrilActionSuccess::Ok)));
        },
        (true, true) => {
            assert!(matches!(file_actual, Ok(TendrilActionSuccess::Skipped)));
            assert!(matches!(dir_actual, Ok(TendrilActionSuccess::Skipped)));
        },
    }

    assert_eq!(setup.local_file_contents(), "Target file contents");
    assert_eq!(
        setup.local_nested_file_contents(),
        "Target nested file contents"
    );
    assert!(setup.local_file.is_symlink());
    assert!(setup.local_dir.is_symlink());
    assert_eq!(setup.td_dir.read_dir().iter().count(), 1);
    if force && !dry_run {
        assert_eq!(setup.remote_file_contents(), "Target file contents");
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

/// See [`crate::tests::common_action_tests::remote_symlink_is_unchanged]
/// for `dry_run` equivalent
#[rstest]
#[case(true)]
#[case(false)]
fn existing_symlinks_at_remote_are_overwritten(#[case] force: bool) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_nested_file();
    setup.make_target_file();
    setup.make_target_nested_file();
    symlink(&setup.remote_file, &setup.target_file, false, true).unwrap();
    symlink(&setup.remote_dir, &setup.target_dir, false, true).unwrap();

    let mut file_tendril = setup.resolved_file_tendril();
    file_tendril.mode = TendrilMode::Link;

    let mut dir_tendril = setup.resolved_dir_tendril();
    dir_tendril.mode = TendrilMode::Link;

    link_tendril(&setup.td_dir, &file_tendril, false, force).unwrap();
    link_tendril(&setup.td_dir, &dir_tendril, false, force).unwrap();

    assert!(setup.remote_file.is_symlink());
    assert!(setup.remote_dir.is_symlink());
    assert_eq!(setup.remote_file_contents(), "Local file contents");
    assert_eq!(
        setup.remote_nested_file_contents(),
        "Local nested file contents"
    );

    assert!(setup.remote_file.is_symlink());
    assert!(setup.remote_dir.is_symlink());
    assert_eq!(setup.remote_file_contents(), "Local file contents");
    assert_eq!(setup.remote_nested_file_contents(), "Local nested file contents");
}

#[rstest]
#[case(true)]
#[case(false)]
fn no_read_access_from_local_file_returns_success(
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

    let tendril = ResolvedTendril::new(
        "NoReadAccess",
        "no_read_access.txt",
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();

    let actual = link_tendril(
        &td_dir,
        &tendril,
        dry_run,
        force,
    );

    assert_eq!(
        temp_parent_dir.path().join("no_read_access.txt").exists(),
        !dry_run
    );
    if dry_run {
        assert!(matches!(actual, Ok(TendrilActionSuccess::Skipped)));
    }
    else {
        assert!(matches!(actual, Ok(TendrilActionSuccess::Ok)));
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn no_read_access_from_local_dir_returns_success(
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

    let tendril = ResolvedTendril::new(
        "NoReadAccess",
        "no_read_access_dir",
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::Link,
    ).unwrap();

    let actual = link_tendril(
        &td_dir,
        &tendril,
        dry_run,
        force,
    );

    assert_eq!(
        temp_parent_dir.path().join("no_read_access_dir").exists(),
        !dry_run
    );
    if dry_run {
        assert!(matches!(actual, Ok(TendrilActionSuccess::Skipped)));
    }
    else {
        assert!(matches!(actual, Ok(TendrilActionSuccess::Ok)));
    }
}

// The symdir equivalent test is not included as both Windows and Unix
// appear to overwrite symdirs regardless of their permissions
#[rstest]
#[case(true)]
#[case(false)]
#[cfg_attr(unix, ignore)] // On most Unix implementations, the symlink permissions
                          // are ignored and the target's permissions are respected
fn no_write_access_at_remote_symfile_returns_io_error_permission_denied_unless_dry_run(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_local_file();
    setup.make_target_file();
    symlink_expose(&setup.remote_file, &setup.target_file, false, true)
        .unwrap();

    // Set file read-only
    let mut perms = metadata(&setup.remote_file).unwrap().permissions();
    perms.set_readonly(true);
    set_permissions(&setup.remote_file, perms).unwrap();

    let mut tendril = setup.resolved_file_tendril();
    tendril.mode = TendrilMode::Link;

    let actual = link_tendril(&setup.td_dir, &tendril, dry_run, force);

    assert_eq!(setup.remote_file_contents(), "Target file contents");
    if dry_run {
        assert!(matches!(actual, Ok(TendrilActionSuccess::Skipped)));
    }
    else {
        match actual {
            Err(TendrilActionError::IoError(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied)
            },
            _ => panic!("{:?}", actual)
        }
    }
}

#[rstest]
fn non_link_mode_tendril_returns_mode_mismatch_error(
    #[values(TendrilMode::DirMerge, TendrilMode::DirOverwrite)]
    mode: TendrilMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    let mut tendril = setup.resolved_file_tendril();
    tendril.mode = mode;

    let actual = link_tendril(&setup.td_dir, &tendril, dry_run, force);

    assert!(matches!(actual, Err(TendrilActionError::ModeMismatch)));
}
