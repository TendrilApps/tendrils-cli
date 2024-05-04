//! Contains tests specific to link actions.
//! See also [`crate::tests::common_action_tests`].

use crate::{
    link_tendril,
    TendrilActionMetadata,
    TendrilMetadata,
};
use crate::enums::{
    FsoType,
    Location,
    TendrilActionError,
    TendrilActionSuccess,
    TendrilMode
};
use crate::tendril::Tendril;
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
#[apply(crate::tests::tendril_tests::valid_groups_and_names)]
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

    let tendril =  Tendril::new(
        "SomeApp",
        name,
        setup.parent_dir.clone(),
        TendrilMode::Link,
    ).unwrap();

    let actual = link_tendril(&setup.td_dir, &tendril, false, force);

    use std::env::consts::FAMILY;
    let expected_target: PathBuf;
    let exp_local_type;
    if as_dir {
        assert_eq!(
            setup.remote_nested_file_contents(),
            "Local nested file contents"
        );
        assert!(setup.remote_dir.is_symlink());
        exp_local_type = Some(FsoType::Dir);
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
        exp_local_type = Some(FsoType::File);
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
    assert_eq!(actual, TendrilActionMetadata {
        md: TendrilMetadata {
            local_type: exp_local_type,
            remote_type: None,
            resolved_path: setup.parent_dir.join(name),
        },
        action_result: Ok(TendrilActionSuccess::New),
    });
}

/// Note: This doesn't apply if the local doesn't exist.
/// See [`local_doesnt_exist_but_td_dir_does_copies_remote_to_local_then_links_unless_dryrun`]
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

    let mut file_tendril = setup.file_tendril();
    file_tendril.mode = TendrilMode::Link;

    let mut dir_tendril = setup.dir_tendril();
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
        },
        (false, true) => {
            exp_file_result = Ok(TendrilActionSuccess::Overwrite);
            exp_dir_result = Ok(TendrilActionSuccess::Overwrite);
        },
        (true, true) => {
            exp_file_result = Ok(TendrilActionSuccess::OverwriteSkipped);
            exp_dir_result = Ok(TendrilActionSuccess::OverwriteSkipped);
        },
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
    assert_eq!(file_actual, TendrilActionMetadata {
        md: TendrilMetadata {
            local_type: Some(FsoType::File),
            remote_type: Some(FsoType::File),
            resolved_path: setup.remote_file,
        },
        action_result: exp_file_result,
    });
    assert_eq!(dir_actual, TendrilActionMetadata {
        md: TendrilMetadata {
            local_type: Some(FsoType::Dir),
            remote_type: Some(FsoType::Dir),
            resolved_path: setup.remote_dir,
        },
        action_result: exp_dir_result, 
    });
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
    symlink_expose(&setup.local_file, &setup.target_file, false, false).unwrap();
    symlink_expose(&setup.local_dir, &setup.target_dir, false, false).unwrap();

    let mut file_tendril = setup.file_tendril();
    file_tendril.mode = TendrilMode::Link;

    let mut dir_tendril = setup.dir_tendril();
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
        },
        (false, true) => {
            exp_file_result = Ok(TendrilActionSuccess::New);
            exp_dir_result = Ok(TendrilActionSuccess::New);
        },
        (true, true) => {
            exp_file_result = Ok(TendrilActionSuccess::NewSkipped);
            exp_dir_result = Ok(TendrilActionSuccess::NewSkipped);
        },
    };

    assert_eq!(file_actual, TendrilActionMetadata {
        md: TendrilMetadata {
            local_type: Some(FsoType::SymFile),
            remote_type: None,
            resolved_path: setup.remote_file.clone(),
        },
        action_result: exp_file_result,
    });
    assert_eq!(dir_actual, TendrilActionMetadata {
        md: TendrilMetadata {
            local_type: Some(FsoType::SymDir),
            remote_type: None,
            resolved_path: setup.remote_dir.clone(),
        },
        action_result: exp_dir_result,
    });
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
    symlink_expose(&setup.remote_file, &setup.target_file, false, true).unwrap();
    symlink_expose(&setup.remote_dir, &setup.target_dir, false, true).unwrap();

    let mut file_tendril = setup.file_tendril();
    file_tendril.mode = TendrilMode::Link;

    let mut dir_tendril = setup.dir_tendril();
    dir_tendril.mode = TendrilMode::Link;

    let file_actual = link_tendril(&setup.td_dir, &file_tendril, false, force);
    let dir_actual = link_tendril(&setup.td_dir, &dir_tendril, false, force);

    assert_eq!(file_actual, TendrilActionMetadata {
        md: TendrilMetadata {
            local_type: Some(FsoType::File),
            remote_type: Some(FsoType::SymFile),
            resolved_path: setup.remote_file.clone(),
        },
        action_result: Ok(TendrilActionSuccess::Overwrite),
    });
    assert_eq!(dir_actual, TendrilActionMetadata {
        md: TendrilMetadata {
            local_type: Some(FsoType::Dir),
            remote_type: Some(FsoType::SymDir),
            resolved_path: setup.remote_dir.clone(),
        },
        action_result: Ok(TendrilActionSuccess::Overwrite),
    });
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

    let tendril = Tendril::new(
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

    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::NewSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::New);
    }
    assert_eq!(actual, TendrilActionMetadata {
        md: TendrilMetadata {
            local_type: Some(FsoType::File),
            remote_type: None,
            resolved_path: temp_parent_dir
                .path()
                .join("no_read_access.txt"),
        },
        action_result: exp_result,
    });
    assert_eq!(
        temp_parent_dir.path().join("no_read_access.txt").exists(),
        !dry_run
    );
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

    let tendril = Tendril::new(
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

    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::NewSkipped);
    }
    else {
        exp_result = Ok(TendrilActionSuccess::New);
    }
    assert_eq!(actual, TendrilActionMetadata {
        md: TendrilMetadata {
            local_type: Some(FsoType::Dir),
            remote_type: None,
            resolved_path: temp_parent_dir
                .path()
                .join("no_read_access_dir"),
        },
        action_result: exp_result,
    });
    assert_eq!(
        temp_parent_dir.path().join("no_read_access_dir").exists(),
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

    let mut tendril = setup.file_tendril();
    tendril.mode = TendrilMode::Link;

    let actual = link_tendril(&setup.td_dir, &tendril, dry_run, force);

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
    assert_eq!(actual, TendrilActionMetadata {
        md: TendrilMetadata {
            local_type: Some(FsoType::File),
            remote_type: Some(FsoType::SymFile),
            resolved_path: setup.remote_file.clone(),
        },
        action_result: exp_result,
    });
    assert_eq!(setup.remote_file_contents(), "Target file contents");
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
    let mut tendril = setup.file_tendril();
    tendril.mode = mode;

    let actual = link_tendril(&setup.td_dir, &tendril, dry_run, force);

    assert_eq!(actual, TendrilActionMetadata {
        md: TendrilMetadata {
            local_type: None,
            remote_type: None,
            resolved_path: setup.remote_file,
        },
        action_result: Err(TendrilActionError::ModeMismatch),
    });
}

/// This is an exception to [`remote_exists_and_is_not_symlink_returns_type_mismatch_error_unless_forced`]
#[rstest]
#[case(true)]
#[case(false)]
fn local_doesnt_exist_but_td_dir_does_copies_remote_to_local_then_links_unless_dryrun(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_td_dir();
    setup.make_remote_file();
    setup.make_remote_nested_file();
    assert!(!setup.local_file.exists());
    assert!(!setup.local_dir.exists());

    let mut file_tendril = setup.file_tendril();
    let mut dir_tendril = setup.dir_tendril();
    file_tendril.mode = TendrilMode::Link;
    dir_tendril.mode = TendrilMode::Link;

    let file_actual = link_tendril(&setup.td_dir, &file_tendril, dry_run, force);
    let dir_actual = link_tendril(&setup.td_dir, &dir_tendril, dry_run, force);

    let exp_result;
    if dry_run {
        exp_result = Ok(TendrilActionSuccess::OverwriteSkipped);
        assert!(!setup.local_file.exists());
        assert!(!setup.local_dir.exists());
    }
    else {
        exp_result = Ok(TendrilActionSuccess::Overwrite);
        assert_eq!(setup.local_file_contents(), "Remote file contents");
        assert_eq!(setup.local_nested_file_contents(), "Remote nested file contents");
    }
    assert_eq!(file_actual, TendrilActionMetadata {
        md: TendrilMetadata {
            local_type: None,
            remote_type: Some(FsoType::File),
            resolved_path: setup.remote_file,
        },
        action_result: exp_result.clone(),
    });
    assert_eq!(dir_actual, TendrilActionMetadata {
        md: TendrilMetadata {
            local_type: None,
            remote_type: Some(FsoType::Dir),
            resolved_path: setup.remote_dir,
        },
        action_result: exp_result,
    });
}
