//! Contains tests specific to link actions.
//! See also [`crate::tests::common_action_tests`].

use crate::test_utils::{
    set_ra,
    symlink_expose,
    Setup,
};
use crate::{
    link_tendril,
    ActionLog,
    FsoType,
    Location,
    Tendril,
    TendrilActionError,
    TendrilActionSuccess,
    TendrilMode,
};
use rstest::rstest;
use std::fs::{create_dir_all, metadata, set_permissions};

/// See also [`crate::tests::common_action_tests::remote_is_unchanged`] for
/// `dry_run` case
#[rstest]
fn local_exists_symlink_to_local_is_created(
    #[values(true, false)] force: bool,
    #[values(true, false)] as_dir: bool,
) {
    let setup = Setup::new();
    let exp_remote_path;
    let mut tendril;
    if as_dir {
        setup.make_local_nested_file();
        exp_remote_path = setup.remote_dir.clone();
        tendril = setup.dir_tendril();
    }
    else {
        setup.make_local_file();
        exp_remote_path = setup.remote_file.clone();
        tendril = setup.file_tendril();
    }
    tendril.mode = TendrilMode::Link;
    assert!(!setup.remote_file.exists());
    assert!(!setup.remote_dir.exists());

    let actual = link_tendril(&tendril, false, force);

    let exp_local_type;
    if as_dir {
        assert_eq!(
            setup.remote_nested_file_contents(),
            "Local nested file contents"
        );
        assert!(setup.remote_dir.is_symlink());
        exp_local_type = Some(FsoType::Dir);
        assert_eq!(
            std::fs::read_link(setup.remote_dir).unwrap(),
            setup.local_dir,
        );
    }
    else {
        assert_eq!(setup.remote_file_contents(), "Local file contents");
        assert!(setup.remote_file.is_symlink());
        exp_local_type = Some(FsoType::File);
        assert_eq!(
            std::fs::read_link(setup.remote_file).unwrap(),
            setup.local_file,
        );
    }
    assert_eq!(
        actual,
        ActionLog::new(
            exp_local_type,
            None,
            exp_remote_path,
            Ok(TendrilActionSuccess::New),
        )
    );
}

/// Note: This doesn't apply if the local doesn't exist.
/// See [`local_doesnt_exist_but_td_repo_does_copies_remote_to_local_then_links_unless_dryrun`]
#[rstest]
#[case(true)]
#[case(false)]
fn remote_exists_and_is_not_symlink_returns_type_mismatch_error_unless_forced(
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_remote_nested_file();
    setup.make_local_file();
    setup.make_local_nested_file();

    let mut file_tendril = setup.file_tendril();
    file_tendril.mode = TendrilMode::Link;

    let mut dir_tendril = setup.dir_tendril();
    dir_tendril.mode = TendrilMode::Link;

    let file_actual =
        link_tendril(&file_tendril, dry_run, force);
    let dir_actual =
        link_tendril(&dir_tendril, dry_run, force);

    let exp_file_result;
    let exp_dir_result;
    match (dry_run, force) {
        (_, false) => {
            exp_file_result = Err(TendrilActionError::TypeMismatch {
                loc: Location::Dest,
                mistype: FsoType::File,
            });
            exp_dir_result = Err(TendrilActionError::TypeMismatch {
                loc: Location::Dest,
                mistype: FsoType::Dir,
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
    assert_eq!(
        file_actual,
        ActionLog::new(
            Some(FsoType::File),
            Some(FsoType::File),
            setup.remote_file,
            exp_file_result,
        )
    );
    assert_eq!(
        dir_actual,
        ActionLog::new(
            Some(FsoType::Dir),
            Some(FsoType::Dir),
            setup.remote_dir,
            exp_dir_result,
        )
    );
}

#[rstest]
fn remote_parent_doesnt_exist_creates_full_parent_structure(
    #[values(true, false)] force: bool,
    #[values(true, false)] as_dir: bool,
) {
    let setup = Setup::new();
    let exp_remote_path;
    let mut tendril;
    if as_dir {
        setup.make_local_subdir_nested_file();
        exp_remote_path = setup.remote_subdir_dir.clone();
        tendril = setup.subdir_dir_tendril();
        assert!(!setup.remote_subdir_dir.parent().unwrap().exists());
    }
    else {
        setup.make_local_subdir_file();
        exp_remote_path = setup.remote_subdir_file.clone();
        tendril = setup.subdir_file_tendril();
        assert!(!setup.remote_subdir_file.parent().unwrap().exists());
    }
    tendril.mode = TendrilMode::Link;

    let actual = link_tendril(&tendril, false, force);

    let exp_local_type;
    if as_dir {
        assert_eq!(
            setup.remote_subdir_nested_file_contents(),
            "Local subdir nested file contents"
        );
        exp_local_type = Some(FsoType::Dir);
        assert_eq!(
            std::fs::read_link(setup.remote_subdir_dir).unwrap(),
            setup.local_subdir_dir,
        );
    }
    else {
        assert_eq!(setup.remote_subdir_file_contents(), "Local subdir file contents");
        exp_local_type = Some(FsoType::File);
        assert_eq!(
            std::fs::read_link(setup.remote_subdir_file).unwrap(),
            setup.local_subdir_file,
        );
    }
    assert_eq!(
        actual,
        ActionLog::new(
            exp_local_type,
            None,
            exp_remote_path,
            Ok(TendrilActionSuccess::New),
        )
    );
}

#[rstest]
#[case(true)]
#[case(false)]
fn local_is_symlink_returns_type_mismatch_error_unless_forced(
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_target_file();
    setup.make_target_nested_file();
    create_dir_all(&setup.group_dir).unwrap();
    symlink_expose(&setup.local_file, &setup.target_file, false, false)
        .unwrap();
    symlink_expose(&setup.local_dir, &setup.target_dir, false, false).unwrap();

    let mut file_tendril = setup.file_tendril();
    file_tendril.mode = TendrilMode::Link;

    let mut dir_tendril = setup.dir_tendril();
    dir_tendril.mode = TendrilMode::Link;

    let file_actual =
        link_tendril(&file_tendril, dry_run, force);
    let dir_actual =
        link_tendril(&dir_tendril, dry_run, force);

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
    assert_eq!(setup.local_file_contents(), "Target file contents");
    assert_eq!(
        setup.local_nested_file_contents(),
        "Target nested file contents"
    );
    assert!(setup.local_file.is_symlink());
    assert!(setup.local_dir.is_symlink());
    assert_eq!(setup.td_repo.read_dir().iter().count(), 1);
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
    symlink_expose(&setup.remote_file, &setup.target_file, false, true)
        .unwrap();
    symlink_expose(&setup.remote_dir, &setup.target_dir, false, true).unwrap();

    let mut file_tendril = setup.file_tendril();
    file_tendril.mode = TendrilMode::Link;

    let mut dir_tendril = setup.dir_tendril();
    dir_tendril.mode = TendrilMode::Link;

    let file_actual =
        link_tendril(&file_tendril, false, force);
    let dir_actual =
        link_tendril(&dir_tendril, false, force);

    assert_eq!(
        file_actual,
        ActionLog::new(
            Some(FsoType::File),
            Some(FsoType::SymFile),
            setup.remote_file.clone(),
            Ok(TendrilActionSuccess::Overwrite),
        )
    );
    assert_eq!(
        dir_actual,
        ActionLog::new(
            Some(FsoType::Dir),
            Some(FsoType::SymDir),
            setup.remote_dir.clone(),
            Ok(TendrilActionSuccess::Overwrite),
        )
    );
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
    assert_eq!(
        setup.remote_nested_file_contents(),
        "Local nested file contents"
    );
}

#[rstest]
#[case(true)]
#[case(false)]
fn no_read_access_from_local_file_returns_success(
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_td_repo_dir();
    setup.make_local_nra_file();

    let tendril = Tendril::new_expose(
        setup.uni_td_repo(),
        "SomeApp/nra.txt".into(),
        setup.remote_nra_file.clone().into(),
        TendrilMode::Link,
    )
    .unwrap();

    let actual = link_tendril(&tendril, dry_run, force);

    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::NewSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::New);
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
    assert_eq!(
        setup.remote_nra_file.exists(),
        !dry_run
    );
}

#[rstest]
#[case(true)]
#[case(false)]
fn no_read_access_from_local_dir_returns_success(
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_td_repo_dir();
    setup.make_local_nra_dir();

    let tendril = Tendril::new_expose(
        setup.uni_td_repo(),
        "SomeApp/nra".into(),
        setup.remote_nra_dir.clone().into(),
        TendrilMode::Link,
    )
    .unwrap();

    let actual = link_tendril(&tendril, dry_run, force);

    set_ra(&setup.local_nra_dir, true);
    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::NewSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::New);
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
    assert_eq!(
        setup.remote_nra_dir.exists(),
        !dry_run
    );
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
    #[values(true, false)] force: bool,
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

    let mut tendril = setup.file_tendril();
    tendril.mode = TendrilMode::Link;

    let actual = link_tendril(&tendril, dry_run, force);

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
            Some(FsoType::SymFile),
            setup.remote_file.clone(),
            exp_result,
        )
    );
    assert_eq!(setup.remote_file_contents(), "Target file contents");
}

#[rstest]
fn non_link_mode_tendril_returns_mode_mismatch_error(
    #[values(TendrilMode::DirMerge, TendrilMode::DirOverwrite)]
    mode: TendrilMode,

    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    let mut tendril = setup.file_tendril();
    tendril.mode = mode;

    let actual = link_tendril(&tendril, dry_run, force);

    assert_eq!(
        actual,
        ActionLog::new(
            None,
            None,
            setup.remote_file,
            Err(TendrilActionError::ModeMismatch),
        )
    );
}

/// This is an exception to
/// [`remote_exists_and_is_not_symlink_returns_type_mismatch_error_unless_forced`]
#[rstest]
#[case(true)]
#[case(false)]
fn local_doesnt_exist_copies_remote_to_local_then_links_unless_dryrun(
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
    #[values(true, false)] repo_exists: bool,
) {
    let setup = Setup::new();
    if repo_exists {
        setup.make_td_repo_dir();
    }
    setup.make_remote_file();
    setup.make_remote_nested_file();
    assert!(!setup.local_file.exists());
    assert!(!setup.local_dir.exists());

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    file_tendril.mode = TendrilMode::Link;
    dir_tendril.mode = TendrilMode::Link;

    let file_actual =
        link_tendril(&file_tendril, dry_run, force);
    let dir_actual =
        link_tendril(&dir_tendril, dry_run, force);

    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::OverwriteSkipped);
        assert!(!setup.local_file.exists());
        assert!(!setup.local_dir.exists());
    }
    else {
        exp_result = Ok(TendrilActionSuccess::Overwrite);
        assert_eq!(setup.local_file_contents(), "Remote file contents");
        assert_eq!(
            setup.local_nested_file_contents(),
            "Remote nested file contents"
        );
    }
    assert_eq!(
        file_actual,
        ActionLog::new(
            None,
            Some(FsoType::File),
            setup.remote_file,
            exp_result.clone(),
        )
    );
    assert_eq!(
        dir_actual,
        ActionLog::new(None, Some(FsoType::Dir), setup.remote_dir, exp_result,)
    );
}
