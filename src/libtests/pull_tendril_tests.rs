use crate::{
    is_tendrils_dir,
    pull_tendril,
    symlink,
    TendrilActionError,
    TendrilActionSuccess,
};
use crate::resolved_tendril::{ResolvedTendril, TendrilMode};
use crate::test_utils::{
    get_disposable_dir,
    get_samples_dir,
    is_empty,
    Setup,
};
use rstest::rstest;
use serial_test::serial;
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
fn remote_exists_copies_successfully(
    #[case] name: String,

    #[values(true, false)]
    force: bool,

    #[values(true, false)]
    as_dir: bool,
) {
    let mut setup = Setup::new();
    setup.remote_file = setup.parent_dir.join(&name);
    setup.remote_dir = setup.parent_dir.join(&name);
    setup.remote_nested_file = setup.remote_dir.join("nested.txt");
    setup.ctrl_file = setup.group_dir.join(&name);
    setup.ctrl_dir = setup.group_dir.join(&name);
    setup.ctrl_nested_file = setup.ctrl_dir.join("nested.txt");
    if as_dir {
        setup.make_remote_dir();
        setup.make_remote_nested_file();
    }
    else {
        setup.make_remote_file();
    }

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        name.clone(),
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    pull_tendril(&setup.td_dir, &tendril, false, force).unwrap();

    if as_dir {
        assert_eq!(setup.ctrl_nested_file_contents(), "Remote nested file contents");
    }
    else {
        assert_eq!(setup.ctrl_file_contents(), "Remote file contents");
    }
    assert_eq!(setup.group_dir.read_dir().iter().count(), 1);
}

#[rstest]
#[case(true)]
#[case(false)]
fn remote_exists_dry_run_returns_skipped_error_does_not_modify_controlled(
    #[case] force: bool,
) {
    // TODO: Test for symlink setup
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_remote_nested_file();
    setup.make_ctrl_file();
    setup.make_ctrl_nested_file();
    let file_tendril = setup.resolved_file_tendril();
    let dir_tendril = setup.resolved_dir_tendril();

    let file_actual = pull_tendril(&setup.td_dir, &file_tendril, true, force);
    let dir_actual = pull_tendril(&setup.td_dir, &dir_tendril, true, force);

    assert!(matches!(file_actual, Ok(TendrilActionSuccess::Skipped)));
    assert!(matches!(dir_actual, Ok(TendrilActionSuccess::Skipped)));
    assert_eq!(setup.ctrl_file_contents(), "Controlled file contents");
    assert_eq!(setup.ctrl_nested_file_contents(), "Controlled nested file contents");
}

// TODO: Test when path is invalid and a copy is attempted (with both a folder AND a file)

#[rstest]
#[case("<mut-testing>", "TendrilsDir", "SomeApp", "misc.txt")]
#[case("Parent", "<mut-testing>", "SomeApp", "misc.txt")]
#[case("Parent", "TendrilsDir", "<mut-testing>", "misc.txt")]
#[case("Parent", "TendrilsDir", "SomeApp", "<mut-testing>")]
#[cfg_attr(windows, ignore)] // These are invalid paths on Windows
#[serial("mut-env-var-testing")]
fn var_in_any_field_exists_uses_raw_path(
    #[case] parent: &str,
    #[case] td_dir: &str,
    #[case] group: &str,
    #[case] name: &str,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    // Any variables should have been resolved at this point
    let mut setup = Setup::new();
    setup.parent_dir = setup.temp_dir.path().join(parent);
    setup.td_dir = setup.temp_dir.path().join(td_dir);
    setup.group_dir = setup.td_dir.join(group);
    setup.remote_file = setup.parent_dir.join(name);
    setup.ctrl_file = setup.group_dir.join(name);
    setup.make_parent_dir();
    setup.make_remote_file();
    setup.make_ctrl_file();
    std::env::set_var("mut-testing", "value");

    let tendril = ResolvedTendril::new(
        group.to_string(),
        name.to_string(),
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    if dry_run {
        assert!(matches!(actual, Ok(TendrilActionSuccess::Skipped)));
        assert_eq!(setup.ctrl_file_contents(), "Controlled file contents");
    }
    else {
        assert!(matches!(actual, Ok(TendrilActionSuccess::Ok)));
        assert_eq!(setup.ctrl_file_contents(), "Remote file contents");
    }
}

#[rstest]
#[case("<I_do_not_exist>", "TendrilsDir", "SomeApp", "misc.txt")]
#[case("Parent", "<I_do_not_exist>", "SomeApp", "misc.txt")]
#[case("Parent", "TendrilsDir", "<I_do_not_exist>", "misc.txt")]
#[case("Parent", "TendrilsDir", "SomeApp", "<I_do_not_exist>")]
#[cfg_attr(windows, ignore)] // These are invalid paths on Windows
fn var_in_any_field_doesnt_exist_uses_raw_path(
    #[case] parent: &str,
    #[case] td_dir: &str,
    #[case] group: &str,
    #[case] name: &str,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let mut setup = Setup::new();
    setup.parent_dir = setup.temp_dir.path().join(parent);
    setup.td_dir = setup.temp_dir.path().join(td_dir);
    setup.group_dir = setup.td_dir.join(group);
    setup.remote_file = setup.parent_dir.join(name);
    setup.ctrl_file = setup.group_dir.join(name);
    setup.make_parent_dir();
    setup.make_remote_file();
    setup.make_ctrl_file();

    let tendril = ResolvedTendril::new(
        group.to_string(),
        name.to_string(),
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    // TODO: Assert error on Windows to allow this test to run on all
    // platforms
    if dry_run {
        assert!(matches!(actual, Ok(TendrilActionSuccess::Skipped)));
        assert_eq!(setup.ctrl_file_contents(), "Controlled file contents");
    }
    else {
        assert!(matches!(actual, Ok(TendrilActionSuccess::Ok)));
        assert_eq!(setup.ctrl_file_contents(), "Remote file contents");
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn remote_doesnt_exist_returns_io_error_not_found(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        PathBuf::from("SomePathThatDoesNotExist"),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    match actual {
        Err(TendrilActionError::IoError(e)) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        _ => panic!(),
    }
    assert!(is_empty(&setup.td_dir));
}

#[rstest]
#[case(true)]
#[case(false)]
fn remote_is_given_td_dir_returns_recursion_error(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "TendrilsDir".to_string(),
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
    assert!(is_empty(&setup.td_dir));
}

#[rstest]
#[case(true)]
#[case(false)]
fn remote_is_ancestor_to_given_td_dir_returns_recursion_error(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let mut setup = Setup::new();
    setup.td_dir = setup.parent_dir  
        .join("Nested1")
        .join("Nested2")
        .join("Nested3")
        .join("TendrilsDir");

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "Nested1".to_string(),
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
    assert!(is_empty(&setup.td_dir));
}

#[rstest]
#[case(true)]
#[case(false)]
fn remote_is_sibling_to_given_td_dir_copies_normally(
    #[case] force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_dir();
    assert_eq!( // Check they are siblings
        setup.remote_dir.parent().unwrap(),
        setup.td_dir.parent().unwrap()
    );
    let tendril = setup.resolved_dir_tendril();

    pull_tendril(&setup.td_dir, &tendril, false, force).unwrap();

    assert!(setup.ctrl_dir.exists());
}

#[rstest]
#[case(true)]
#[case(false)]
fn remote_is_direct_child_of_given_td_dir_returns_recursion_error(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_td_dir = TempDir::new_in(
        get_disposable_dir(),
        "TendrilsDir"
    ).unwrap();
    let td_dir = temp_td_dir.path().to_path_buf();
    let parent_dir = td_dir.clone();
    let source = parent_dir.join("misc.txt");
    write(&source, "Remote file contents").unwrap();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(
        &td_dir,
        &tendril,
        dry_run,
        force,
    );

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
    assert_eq!(read_to_string(source).unwrap(), "Remote file contents");
    assert!(td_dir.read_dir().unwrap().into_iter().count() == 1);
}

#[rstest]
#[case(true)]
#[case(false)]
fn remote_is_nested_child_of_given_td_dir_returns_recursion_error(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_td_dir = TempDir::new_in(
        get_disposable_dir(),
        "TendrilsDir"
    ).unwrap();
    let td_dir = temp_td_dir.path().to_path_buf();
    let parent_dir = td_dir
        .join("Nested1")
        .join("Nested2")
        .join("Nested3");
    let source = parent_dir
        .join("misc.txt");
    create_dir_all(&source.parent().unwrap()).unwrap();
    write(&source, "Remote file contents").unwrap();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(&td_dir, &tendril, dry_run, force);

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
    assert_eq!(read_to_string(source).unwrap(), "Remote file contents");
    assert!(td_dir.read_dir().unwrap().into_iter().count() == 1);
}

#[rstest]
#[case(true)]
#[case(false)]
fn remote_is_another_td_dir_still_copies(#[case] force: bool) {
    let setup = Setup::new();
    setup.make_remote_nested_file();
    write(&setup.remote_dir.join("tendrils.json"), "").unwrap();
    assert!(is_tendrils_dir(&setup.remote_dir));

    let tendril = setup.resolved_dir_tendril();

    pull_tendril(&setup.td_dir, &tendril, false, force).unwrap();

    assert!(setup.ctrl_dir.join("tendrils.json").exists());
    assert!(setup.ctrl_nested_file.exists());
    assert_eq!(setup.ctrl_dir.read_dir().unwrap().count(), 2);
}

#[rstest]
#[case(TendrilMode::DirMerge)]
#[case(TendrilMode::DirOverwrite)]
fn remote_is_file_and_ctrl_is_dir_returns_type_mismatch_error_unless_forced(
    #[case] mode: TendrilMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    create_dir_all(&setup.ctrl_file).unwrap();

    let mut tendril = setup.resolved_file_tendril();
    tendril.mode = mode;

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    match (dry_run, force) {
        (_, false) => {
            assert!(matches!(actual, Err(TendrilActionError::TypeMismatch)));
        },
        (false, true) => {
            assert!(matches!(actual, Ok(TendrilActionSuccess::Ok)));
        },
        (true, true) => {
            assert!(matches!(actual, Ok(TendrilActionSuccess::Skipped)));
        },
    }

    if force && !dry_run {
        assert_eq!(setup.remote_file_contents(), "Remote file contents");
        assert_eq!(setup.ctrl_file_contents(), "Remote file contents");
    }
    else {
        assert_eq!(setup.remote_file_contents(), "Remote file contents");
        assert!(setup.ctrl_file.is_dir());
        assert!(is_empty(&setup.ctrl_file));
    }
}

#[rstest]
#[case(TendrilMode::DirMerge)]
#[case(TendrilMode::DirOverwrite)]
fn remote_is_dir_and_ctrl_is_file_returns_type_mismatch_error_unless_forced(
    #[case] mode: TendrilMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_nested_file();
    setup.make_group_dir();
    write(&setup.ctrl_dir, "I'm a file!").unwrap();

    let mut tendril = setup.resolved_dir_tendril();
    tendril.mode = mode;

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    match (dry_run, force) {
        (_, false) => {
            assert!(matches!(actual, Err(TendrilActionError::TypeMismatch)));
        },
        (false, true) => {
            assert!(matches!(actual, Ok(TendrilActionSuccess::Ok)));
        },
        (true, true) => {
            assert!(matches!(actual, Ok(TendrilActionSuccess::Skipped)));
        },
    }

    assert!(setup.remote_dir.is_dir());
    if force && !dry_run {
        assert_eq!(&setup.ctrl_nested_file_contents(), "Remote nested file contents");
        assert_eq!(setup.remote_dir.read_dir().iter().count(), 1);
        assert_eq!(setup.ctrl_dir.read_dir().iter().count(), 1);
    }
    else {
        let ctrl_dir_contents = read_to_string(&setup.ctrl_dir).unwrap();
        assert_eq!(ctrl_dir_contents, "I'm a file!");
        assert_eq!(setup.remote_dir.read_dir().iter().count(), 1);
        assert_eq!(setup.td_dir.read_dir().iter().count(), 1);
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn remote_is_symlink_returns_type_mismatch_error_unless_forced_then_copies_symlink_target_contents_keeps_source_name(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_target_file();
    setup.make_target_nested_file();
    symlink(&setup.remote_file, &setup.target_file, false, false).unwrap();
    symlink(&setup.remote_dir, &setup.target_dir, false, false).unwrap();

    let file_tendril = setup.resolved_file_tendril();
    let dir_tendril = setup.resolved_dir_tendril();

    let file_actual = pull_tendril(
        &setup.td_dir,
        &file_tendril,
        dry_run,
        force,
    );
    let dir_actual = pull_tendril(
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

    assert!(setup.remote_file.is_symlink());
    assert!(setup.remote_dir.is_symlink());
    assert!(!setup.ctrl_file.is_symlink());
    assert!(!setup.ctrl_dir.is_symlink());
    if force && !dry_run {
        assert_eq!(setup.ctrl_file.file_name().unwrap(), "misc.txt");
        assert_eq!(setup.ctrl_file_contents(), "Target file contents");
        assert_eq!(setup.ctrl_dir.file_name().unwrap(), "misc");
        assert_eq!(
            setup.ctrl_nested_file_contents(),
            "Target nested file contents"
        );
    }
    else {
        assert!(is_empty(&setup.td_dir));
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn ctrl_is_symlink_returns_type_mismatch_error_unless_forced(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_remote_nested_file();
    setup.make_target_file();
    setup.make_target_nested_file();
    setup.make_group_dir();
    symlink(&setup.ctrl_file, &setup.target_file, false, true).unwrap();
    symlink(&setup.ctrl_dir, &setup.target_dir, false, true).unwrap();

    let file_tendril = setup.resolved_file_tendril();
    let dir_tendril = setup.resolved_dir_tendril();

    let file_actual = pull_tendril(
        &setup.td_dir,
        &file_tendril,
        dry_run,
        force,
    );
    let dir_actual = pull_tendril(
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
        assert!(!setup.ctrl_file.is_symlink());
        assert!(!setup.ctrl_dir.is_symlink());
        assert_eq!(setup.ctrl_file_contents(), "Remote file contents");
        assert_eq!(setup.ctrl_nested_file_contents(), "Remote nested file contents");
    }
    else {
        assert!(setup.ctrl_file.is_symlink());
        assert!(setup.ctrl_dir.is_symlink());
        assert_eq!(setup.ctrl_file_contents(), "Target file contents");
        assert_eq!(setup.ctrl_nested_file_contents(), "Target nested file contents");
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn no_read_access_from_remote_file_returns_io_error_permission_denied(
    #[case] force: bool,
) {
    let temp_td_dir = TempDir::new_in(
        get_disposable_dir(),
        "TendrilsDir"
    ).unwrap();

    // Note: This test sample is not version controlled and must first
    // be created using the setup script - See dev/setup-tendrils.nu
    let given_parent_dir = get_samples_dir().join("NoReadAccess");

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "no_read_access.txt".to_string(),
        given_parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(
        &temp_td_dir.path(),
        &given,
        false,
        force,
    );

    match actual {
        Err(TendrilActionError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied)
        },
        _ => panic!()
    }
    assert!(is_empty(&temp_td_dir.path().join("SomeApp")));
}

#[rstest]
#[case(true)]
#[case(false)]
fn no_read_access_from_remote_dir_returns_io_error_permission_denied(
    #[case] force: bool,
) {
    let temp_td_dir = TempDir::new_in(
        get_disposable_dir(),
        "TendrilsDir"
    ).unwrap();
    let given_parent_dir = get_samples_dir().join("NoReadAccess");

    // Note: This test sample is not version controlled and must first
    // be created using the setup script - See dev/setup-tendrils.nu
    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "no_read_access_dir".to_string(),
        given_parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(
        &temp_td_dir.path(),
        &given,
        false,
        force,
    );

    match actual {
        Err(TendrilActionError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied)
        },
        _ => panic!()
    }
    assert!(is_empty(&temp_td_dir.path().join("SomeApp")));
}

#[rstest]
#[case(true)]
#[case(false)]
fn no_write_access_at_ctrl_file_returns_io_error_permission_denied(
    #[case] force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_ctrl_file();

    // Set file read-only
    let mut perms = metadata(&setup.ctrl_file).unwrap().permissions();
    perms.set_readonly(true);
    set_permissions(&setup.ctrl_file, perms).unwrap();

    let tendril = setup.resolved_file_tendril();

    let actual = pull_tendril(&setup.td_dir, &tendril, false, force);

    match actual {
        Err(TendrilActionError::IoError(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied)
        },
        _ => panic!()
    }
    assert_eq!(setup.ctrl_file_contents(), "Controlled file contents");
}

// TODO: No write access at dest_dir?

#[rstest]
#[case(TendrilMode::DirMerge)]
#[case(TendrilMode::DirOverwrite)]
fn file_tendril_overwrites_ctrl_file_regardless_of_dir_merge_mode(
    #[case] mode: TendrilMode,
    
    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_ctrl_file();

    let mut tendril = setup.resolved_file_tendril();
    tendril.mode = mode;

    pull_tendril(&setup.td_dir, &tendril, false, force).unwrap();

    assert_eq!(setup.ctrl_file_contents(), "Remote file contents");
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_overwrite_w_dir_tendril_replaces_ctrl_dir_recursively(
    #[case] force: bool,
) {
    let setup = Setup::new();
    let remote_nested_dir = &setup.remote_dir.join("NestedDir");
    let remote_new_2nested_file = remote_nested_dir.join("new_nested.txt");
    let ctrl_nested_dir = &setup.ctrl_dir.join("NestedDir");
    let ctrl_new_2nested_file = ctrl_nested_dir.join("new_nested.txt");
    let ctrl_extra_2nested_file = ctrl_nested_dir.join("extra_nested.txt");
    setup.make_remote_nested_file();
    setup.make_ctrl_nested_file();
    create_dir_all(&remote_nested_dir).unwrap();
    create_dir_all(&ctrl_nested_dir).unwrap();
    write(&remote_new_2nested_file, "I'm not in the tendrils dir").unwrap();
    write(&ctrl_extra_2nested_file, "I'm not in the source dir").unwrap();

    let mut tendril = setup.resolved_dir_tendril();
    tendril.mode = TendrilMode::DirOverwrite;

    pull_tendril(&setup.td_dir, &tendril, false, force).unwrap();

    let ctrl_new_2nested_file_contents = read_to_string(ctrl_new_2nested_file).unwrap();
    assert_eq!(setup.ctrl_nested_file_contents(), "Remote nested file contents");
    assert_eq!(ctrl_new_2nested_file_contents, "I'm not in the tendrils dir");
    assert!(!ctrl_extra_2nested_file.exists());
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_merge_w_dir_tendril_merges_w_ctrl_dir_recursively(
    #[case] force: bool,
) {
    let setup = Setup::new();
    let remote_nested_dir = &setup.remote_dir.join("NestedDir");
    let remote_new_2nested_file = remote_nested_dir.join("new_nested.txt");
    let ctrl_nested_dir = &setup.ctrl_dir.join("NestedDir");
    let ctrl_new_2nested_file = ctrl_nested_dir.join("new_nested.txt");
    let ctrl_extra_2nested_file = ctrl_nested_dir.join("extra_nested.txt");
    setup.make_remote_nested_file();
    setup.make_ctrl_nested_file();
    create_dir_all(&remote_nested_dir).unwrap();
    create_dir_all(&ctrl_nested_dir).unwrap();
    write(&remote_new_2nested_file, "I'm not in the tendrils dir").unwrap();
    write(&ctrl_extra_2nested_file, "I'm not in the source dir").unwrap();

    let mut tendril = setup.resolved_dir_tendril();
    tendril.mode = TendrilMode::DirMerge;

    pull_tendril(&setup.td_dir, &tendril, false, force).unwrap();

    let ctrl_new_2nested_file_contents = read_to_string(ctrl_new_2nested_file).unwrap();
    let ctrl_extra_2nested_file_contents = read_to_string(ctrl_extra_2nested_file).unwrap();
    assert_eq!(setup.ctrl_nested_file_contents(), "Remote nested file contents");
    assert_eq!(ctrl_new_2nested_file_contents, "I'm not in the tendrils dir");
    assert_eq!(ctrl_extra_2nested_file_contents, "I'm not in the source dir");
}

#[rstest]
#[case(true)]
#[case(false)]
fn td_dir_doesnt_exist_creates_dir_and_subdirs_first_except_if_dry_run(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();

    let tendril = setup.resolved_file_tendril();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    if dry_run {
        assert!(matches!(actual, Ok(TendrilActionSuccess::Skipped)));
        assert!(!setup.td_dir.exists());
    }
    else {
        assert!(matches!(actual, Ok(TendrilActionSuccess::Ok)));
        assert_eq!(setup.ctrl_file_contents(), "Remote file contents");
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn remote_file_is_unchanged(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();

    let tendril = setup.resolved_file_tendril();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    assert_eq!(setup.remote_file_contents(), "Remote file contents");
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
fn remote_dir_is_unchanged(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_nested_file();

    let tendril = setup.resolved_dir_tendril();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    assert_eq!(setup.remote_nested_file_contents(), "Remote nested file contents");
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
fn other_tendrils_in_same_group_dir_are_unchanged(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    let some_other_ctrl_file= &setup.group_dir.join("other.txt");
    setup.make_group_dir();
    write(some_other_ctrl_file, "Another tendril from the same group").unwrap();

    let tendril = setup.resolved_file_tendril();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    // Check that other tendril is unchanged
    let some_other_ctrl_file_contents = read_to_string(some_other_ctrl_file).unwrap();
    assert_eq!(some_other_ctrl_file_contents, "Another tendril from the same group");
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
fn link_mode_tendril_returns_mode_mismatch_error(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();

    let mut tendril = setup.resolved_file_tendril();
    tendril.mode = TendrilMode::Link;

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    assert!(matches!(actual, Err(TendrilActionError::ModeMismatch)));
    assert_eq!(&setup.remote_file_contents(), "Remote file contents");
    assert!(is_empty(&setup.td_dir));
}
