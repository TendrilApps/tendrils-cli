use crate::{
    is_tendrils_dir,
    pull_tendril,
    symlink,
    TendrilActionError,
};
use crate::resolved_tendril::{ResolvedTendril, TendrilMode};
use crate::test_utils::{
    get_disposable_dir,
    get_samples_dir,
    get_username_can_panic,
    is_empty,
    Setup,
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
fn tendril_exists_at_source_path_copies_successfully(
    #[case] name: String,

    #[values(true, false)]
    force: bool,

    #[values(true, false)]
    as_dir: bool,
) {
    let mut setup = Setup::new();
    setup.local_file = setup.parent_dir.join(&name);
    setup.local_dir = setup.parent_dir.join(&name);
    setup.local_nested_file = setup.local_dir.join("nested.txt");
    setup.ctrl_file = setup.group_dir.join(&name);
    setup.ctrl_dir = setup.group_dir.join(&name);
    setup.ctrl_nested_file = setup.ctrl_dir.join("nested.txt");
    if as_dir {
        setup.make_local_dir();
        setup.make_local_nested_file();
    }
    else {
        setup.make_local_file();
    }

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        name.clone(),
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    pull_tendril(&setup.td_dir, &tendril, false, force).unwrap();

    if as_dir {
        assert_eq!(setup.ctrl_nested_file_contents(), "Local nested file contents");
    }
    else {
        assert_eq!(setup.ctrl_file_contents(), "Local file contents");
    }
    assert_eq!(setup.group_dir.read_dir().iter().count(), 1);
}

#[rstest]
#[case(true)]
#[case(false)]
fn tendril_exists_at_source_path_in_dry_run_returns_skipped_error_does_not_modify_dest(
    #[case] force: bool,
) {
    // TODO: Test for symlink setup
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_nested_file();
    setup.make_ctrl_file();
    setup.make_ctrl_nested_file();

    let file_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let dir_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc".to_string(),
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let file_actual = pull_tendril(&setup.td_dir, &file_tendril, true, force);
    let dir_actual = pull_tendril(&setup.td_dir, &dir_tendril, true, force);

    assert!(matches!(file_actual, Err(TendrilActionError::Skipped)));
    assert!(matches!(dir_actual, Err(TendrilActionError::Skipped)));
    assert_eq!(setup.ctrl_file_contents(), "Controlled file contents");
    assert_eq!(setup.ctrl_nested_file_contents(), "Controlled nested file contents");
}

// TODO: Test when path is invalid and a copy is attempted (with both a folder AND a file)

#[rstest]
#[case("TendrilsDir", "SomeApp", "<user>")]
#[case("TendrilsDir", "<user>", "misc.txt")]
#[case("<user>", "SomeApp", "misc.txt")]
#[cfg(not(windows))] // These are invalid paths on Windows
fn supported_var_in_td_dir_or_group_or_name_uses_raw_path(
    #[case] td_dir: &str,
    #[case] group: &str,
    #[case] name: &str,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let mut setup = Setup::new();
    setup.td_dir = setup.temp_dir.path().join(td_dir);
    setup.group_dir = setup.td_dir.join(group);
    setup.local_file = setup.parent_dir.join(name);
    setup.ctrl_file = setup.group_dir.join(name);
    setup.make_local_file();
    setup.make_ctrl_file();

    let tendril = ResolvedTendril::new(
        group.to_string(),
        name.to_string(),
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    if dry_run {
        assert!(matches!(actual, Err(TendrilActionError::Skipped)));
        assert_eq!(setup.ctrl_file_contents(), "Controlled file contents");
    }
    else {
        assert!(matches!(actual, Ok(())));
        assert_eq!(setup.ctrl_file_contents(), "Local file contents");
    }
}

#[rstest]
#[case("Parent<>Dir")]
#[case("Parent<unsupported>Dir")]
#[case("<unsupported>")]
#[cfg(not(windows))] // These are invalid paths on Windows
fn unsupported_var_in_parent_path_uses_raw_path(
    #[case] parent_name_raw: &str,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let mut setup = Setup::new();
    setup.parent_dir = setup.temp_dir.path().join(parent_name_raw);
    setup.local_file = setup.parent_dir.join("misc.txt");
    create_dir_all(&setup.parent_dir).unwrap();
    setup.make_local_file();
    setup.make_ctrl_file();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    if dry_run {
        assert!(matches!(actual, Err(TendrilActionError::Skipped)));
        assert_eq!(setup.ctrl_file_contents(), "Controlled file contents");
    }
    else {
        assert!(matches!(actual, Ok(())));
        assert_eq!(setup.ctrl_file_contents(), "Local file contents");
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_doesnt_exist_returns_io_error_not_found(
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
fn source_is_given_td_dir_returns_recursion_error(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    // TODO: Change to setup
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let given_td_dir = temp_parent_dir
        .path()
        .join("TendrilsDir");
    let given_parent_dir = temp_parent_dir.path().to_path_buf();
    create_dir_all(&temp_parent_dir.path().join(get_username_can_panic())).unwrap();

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "TendrilsDir".to_string(),
        given_parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(
        &given_td_dir,
        &given,
        dry_run,
        force,
    );

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
    assert!(is_empty(&given_td_dir));
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_is_ancestor_to_given_td_dir_returns_recursion_error(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let given_parent_dir = temp_parent_dir.path().to_path_buf();
    let given_td_dir = given_parent_dir
        .join("Nested1")
        .join("Nested2")
        .join("Nested3")
        .join("TendrilsDir");
    create_dir_all(
        &temp_parent_dir.path().join(get_username_can_panic())
    ).unwrap();

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "Nested1".to_string(),
        given_parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(
        &given_td_dir,
        &given,
        dry_run,
        force,
    );

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
    assert!(is_empty(&given_td_dir));
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_is_sibling_to_given_td_dir_copies_normally(
    #[case] force: bool,
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let given_parent_dir = temp_parent_dir.path().to_path_buf();
    let given_td_dir = given_parent_dir
        .join("TendrilsDir");
    create_dir_all(&given_parent_dir
        .join("SiblingDir")
    ).unwrap();

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "SiblingDir".to_string(),
        given_parent_dir,
        TendrilMode::DirOverwrite
    ).unwrap();

    pull_tendril(&given_td_dir, &given, false, force).unwrap();

    assert!(given_td_dir
        .join("SomeApp")
        .join("SiblingDir")
        .exists()
    );
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_is_direct_child_of_given_td_dir_returns_recursion_error(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_td_dir = TempDir::new_in(
        get_disposable_dir(),
        "TendrilsDir"
    ).unwrap();
    let given_td_dir = temp_td_dir.path().to_path_buf();
    let given_parent_dir = given_td_dir.clone();
    let source = given_td_dir.join("misc.txt");
    create_dir_all(&given_td_dir).unwrap();
    write(&source, "Source file contents").unwrap();

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        given_parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(
        &given_td_dir,
        &given,
        dry_run,
        force,
    );

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
    assert_eq!(read_to_string(source).unwrap(), "Source file contents");
    assert!(given_td_dir.read_dir().unwrap().into_iter().count() == 1);
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_is_nested_child_of_given_td_dir_returns_recursion_error(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_td_dir = TempDir::new_in(
        get_disposable_dir(),
        "TendrilsDir"
    ).unwrap();
    let given_td_dir = temp_td_dir.path().to_path_buf();
    let given_parent_dir = given_td_dir
        .join("Nested1")
        .join("Nested2")
        .join("Nested3");
    let source = given_parent_dir
        .join("misc.txt");
    create_dir_all(&source.parent().unwrap()).unwrap();
    write(&source, "Source file contents").unwrap();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        given_parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(
        &given_td_dir,
        &tendril,
        dry_run,
        force,
    );

    assert!(matches!(actual, Err(TendrilActionError::Recursion)));
    assert_eq!(read_to_string(source).unwrap(), "Source file contents");
    assert!(given_td_dir.read_dir().unwrap().into_iter().count() == 1);
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_is_another_td_dir_still_copies(#[case] force: bool) {
    let setup = Setup::new();
    setup.make_local_nested_file();
    write(&setup.local_dir.join("tendrils.json"), "").unwrap();
    assert!(is_tendrils_dir(&setup.local_dir));

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc".to_string(),
        setup.parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();

    pull_tendril(&setup.td_dir, &tendril, false, force).unwrap();

    assert!(setup.ctrl_dir.join("tendrils.json").exists());
    assert!(setup.ctrl_nested_file.exists());
    assert_eq!(setup.ctrl_dir.read_dir().unwrap().count(), 2);
}

#[rstest]
#[case(TendrilMode::DirMerge)]
#[case(TendrilMode::DirOverwrite)]
fn source_is_file_and_dest_is_dir_returns_type_mismatch_error_unless_forced(
    #[case] mode: TendrilMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    create_dir_all(&setup.ctrl_file).unwrap();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        setup.parent_dir.clone(),
        mode,
    ).unwrap();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    match (dry_run, force) {
        (_, false) => {
            assert!(matches!(actual, Err(TendrilActionError::TypeMismatch)));
        },
        (false, true) => {
            assert!(matches!(actual, Ok(())));
        },
        (true, true) => {
            assert!(matches!(actual, Err(TendrilActionError::Skipped)));
        },
    }

    if force && !dry_run {
        assert_eq!(setup.local_file_contents(), "Local file contents");
        assert_eq!(setup.ctrl_file_contents(), "Local file contents");
    }
    else {
        assert_eq!(setup.local_file_contents(), "Local file contents");
        assert!(setup.ctrl_file.is_dir());
        assert!(is_empty(&setup.ctrl_file));
    }
}

#[rstest]
#[case(TendrilMode::DirMerge)]
#[case(TendrilMode::DirOverwrite)]
fn source_is_dir_and_dest_is_file_returns_type_mismatch_error_unless_forced(
    #[case] mode: TendrilMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_local_nested_file();
    setup.make_group_dir();
    write(&setup.ctrl_dir, "I'm a file!").unwrap();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc".to_string(),
        setup.parent_dir.clone(),
        mode,
    ).unwrap();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    match (dry_run, force) {
        (_, false) => {
            assert!(matches!(actual, Err(TendrilActionError::TypeMismatch)));
        },
        (false, true) => {
            assert!(matches!(actual, Ok(())));
        },
        (true, true) => {
            assert!(matches!(actual, Err(TendrilActionError::Skipped)));
        },
    }

    assert!(setup.local_dir.is_dir());
    if force && !dry_run {
        assert_eq!(&setup.ctrl_nested_file_contents(), "Local nested file contents");
        assert_eq!(setup.local_dir.read_dir().iter().count(), 1);
        assert_eq!(setup.ctrl_dir.read_dir().iter().count(), 1);
    }
    else {
        let ctrl_dir_contents = read_to_string(&setup.ctrl_dir).unwrap();
        assert_eq!(ctrl_dir_contents, "I'm a file!");
        assert_eq!(setup.local_dir.read_dir().iter().count(), 1);
        assert_eq!(setup.td_dir.read_dir().iter().count(), 1);
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn source_is_symlink_returns_type_mismatch_error_unless_forced_then_copies_symlink_target(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();

    let given_parent_dir = temp_parent_dir.path().to_path_buf();
    let given_td_dir = given_parent_dir.join("TendrilsDir");
    let source_file = given_parent_dir.join("symfile.txt");
    let source_dir = given_parent_dir.join("symdir");
    let source_nested_file = source_dir.join("nested.txt");
    let dest_file = given_td_dir.join("SomeApp").join("symfile.txt");
    let dest_dir = given_td_dir.join("SomeApp").join("symdir");
    let target_file = given_parent_dir.join("target.txt");
    let target_dir = given_parent_dir.join("target_dir");
    let target_nested_file = target_dir.join("nested.txt");
    create_dir_all(&source_dir).unwrap();
    write(&source_file, "Source file contents").unwrap();
    write(&source_nested_file, "Source nested file contents").unwrap();
    create_dir_all(&target_dir).unwrap();
    write(&target_file, "Target file contents").unwrap();
    write(&target_nested_file, "Target nested file contents").unwrap();
    symlink(&source_file, &target_file, false, false).unwrap();
    symlink(&source_dir, &target_dir, false, false).unwrap();

    let file_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "symfile.txt".to_string(),
        given_parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();
    let dir_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "symdir".to_string(),
        given_parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();

    let file_actual = pull_tendril(
        &given_td_dir,
        &file_tendril,
        dry_run,
        force,
    );
    let dir_actual = pull_tendril(
        &given_td_dir,
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
            assert!(matches!(file_actual, Ok(())));
            assert!(matches!(dir_actual, Ok(())));
        },
        (true, true) => {
            assert!(matches!(file_actual, Err(TendrilActionError::Skipped)));
            assert!(matches!(dir_actual, Err(TendrilActionError::Skipped)));
        },
    }

    assert!(!dest_file.is_symlink());
    assert!(!dest_dir.is_symlink());

    if force && !dry_run {
        let dest_file_contents = read_to_string(&dest_file).unwrap();
        let dest_nested_contents = read_to_string(dest_dir.join("nested.txt")).unwrap();
        assert_eq!(dest_file_contents, "Target file contents");
        assert_eq!(dest_nested_contents, "Target nested file contents");
    }
    else {
        assert!(source_file.is_symlink());
        assert!(source_dir.is_symlink());
        assert!(is_empty(&given_td_dir));
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn dest_is_symlink_returns_type_mismatch_error_unless_forced(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_source_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();

    let given_parent_dir = temp_source_dir.path().to_path_buf();
    let given_td_dir = given_parent_dir.join("TendrilsDir");
    let source_file = given_parent_dir.join("symfile.txt");
    let source_dir = given_parent_dir.join("symdir");
    let source_nested_file = source_dir.join("nested.txt");
    let dest_file = given_td_dir.join("SomeApp").join("symfile.txt");
    let dest_dir = given_td_dir.join("SomeApp").join("symdir");
    let target_file = given_parent_dir.join("target.txt");
    let target_dir = given_parent_dir.join("target_dir");
    let target_nested_file = target_dir.join("nested.txt");
    create_dir_all(&source_dir).unwrap();
    write(&source_file, "Source file contents").unwrap();
    write(&source_nested_file, "Source nested file contents").unwrap();
    create_dir_all(&target_dir).unwrap();
    write(&target_file, "Target file contents").unwrap();
    write(&target_nested_file, "Target nested file contents").unwrap();
    create_dir_all(&given_td_dir.join("SomeApp")).unwrap();
    symlink(&dest_file, &target_file, false, false).unwrap();
    symlink(&dest_dir, &target_dir, false, false).unwrap();

    let file_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "symfile.txt".to_string(),
        given_parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();
    let dir_tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "symdir".to_string(),
        given_parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();

    let file_actual = pull_tendril(
        &given_td_dir,
        &file_tendril,
        dry_run,
        force,
    );
    let dir_actual = pull_tendril(
        &given_td_dir,
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
            assert!(matches!(file_actual, Ok(())));
            assert!(matches!(dir_actual, Ok(())));
        },
        (true, true) => {
            assert!(matches!(file_actual, Err(TendrilActionError::Skipped)));
            assert!(matches!(dir_actual, Err(TendrilActionError::Skipped)));
        },
    }

    let dest_file_contents = read_to_string(&dest_file).unwrap();
    let dest_nested_contents = read_to_string(dest_dir.join("nested.txt")).unwrap();
    if force && !dry_run {
        assert!(!dest_file.is_symlink());
        assert!(!dest_dir.is_symlink());
        assert_eq!(dest_file_contents, "Source file contents");
        assert_eq!(dest_nested_contents, "Source nested file contents");
    }
    else {
        assert!(dest_file.is_symlink());
        assert!(dest_dir.is_symlink());
        assert_eq!(dest_file_contents, "Target file contents");
        assert_eq!(dest_nested_contents, "Target nested file contents");
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn no_read_access_from_source_file_returns_io_error_permission_denied(
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
fn no_read_access_from_source_dir_returns_io_error_permission_denied(
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
fn no_write_access_at_dest_file_returns_io_error_permission_denied(
    #[case] force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_ctrl_file();

    // Set file read-only
    let mut perms = metadata(&setup.ctrl_file).unwrap().permissions();
    perms.set_readonly(true);
    set_permissions(&setup.ctrl_file, perms).unwrap();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();

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
fn file_tendril_overwrites_dest_file_regardless_of_dir_merge_mode(
    #[case] mode: TendrilMode,
    
    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_ctrl_file();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        setup.parent_dir.clone(),
        mode,
    ).unwrap();

    pull_tendril(&setup.td_dir, &tendril, false, force).unwrap();

    assert_eq!(setup.ctrl_file_contents(), "Local file contents");
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_merge_false_w_dir_tendril_overwrites_dest_dir_recursively(
    #[case] force: bool,
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let given_td_dir = &temp_parent_dir.path().join("TendrilsDir");
    let source= &temp_parent_dir.path().join("SourceDir");
    let nested_dir= &source.join("NestedDir");
    let source_misc_file = source.join("misc.txt");
    let source_nested_file = nested_dir.join("nested.txt");
    let source_new_nested_file = nested_dir.join("new_nested.txt");
    let dest_misc_file = given_td_dir
        .join("SomeApp")
        .join("SourceDir")
        .join("misc.txt");
    let dest_nested_file = given_td_dir
        .join("SomeApp")
        .join("SourceDir")
        .join("NestedDir")
        .join("nested.txt");
    let dest_new_nested_file = given_td_dir
        .join("SomeApp")
        .join("SourceDir")
        .join("NestedDir")
        .join("new_nested.txt");
    let dest_extra_nested_file = given_td_dir
        .join("SomeApp")
        .join("SourceDir")
        .join("NestedDir")
        .join("extra_nested.txt"); // Should no longer exist
    create_dir_all(&nested_dir).unwrap();
    create_dir_all(dest_nested_file.parent().unwrap()).unwrap();
    write(&source_misc_file, "Source misc file").unwrap();
    write(&source_nested_file, "Source nested file").unwrap();
    write(&source_new_nested_file, "I'm not in the tendrils dir").unwrap();
    write(&dest_misc_file, "Existing misc file").unwrap();
    write(&dest_nested_file, "Existing nested file").unwrap();
    write(&dest_extra_nested_file, "I'm not in the source dir").unwrap();

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "SourceDir".to_string(),
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    pull_tendril(&given_td_dir, &given, false, force).unwrap();

    let dest_misc_contents = read_to_string(dest_misc_file).unwrap();
    let dest_nested_contents = read_to_string(dest_nested_file).unwrap();
    let dest_new_nested_contents = read_to_string(dest_new_nested_file).unwrap();
    assert_eq!(dest_misc_contents, "Source misc file");
    assert_eq!(dest_nested_contents, "Source nested file");
    assert_eq!(dest_new_nested_contents, "I'm not in the tendrils dir");
    assert!(!dest_extra_nested_file.exists());
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_merge_true_w_dir_tendril_merges_w_dest_dir_recursively(
    #[case] force: bool,
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let given_td_dir = &temp_parent_dir.path().join("TendrilsDir");
    let source= &temp_parent_dir.path().join("SourceDir");
    let nested_dir= &source.join("NestedDir");
    let source_misc_file = source.join("misc.txt");
    let source_nested_file = nested_dir.join("nested.txt");
    let source_new_nested_file = nested_dir.join("new_nested.txt");
    let dest_misc_file = given_td_dir
        .join("SomeApp")
        .join("SourceDir")
        .join("misc.txt");
    let dest_nested_file = given_td_dir
        .join("SomeApp")
        .join("SourceDir")
        .join("NestedDir")
        .join("nested.txt");
    let dest_new_nested_file = given_td_dir
        .join("SomeApp")
        .join("SourceDir")
        .join("NestedDir")
        .join("new_nested.txt");
    let dest_extra_nested_file = given_td_dir
        .join("SomeApp")
        .join("SourceDir")
        .join("NestedDir")
        .join("extra_nested.txt");
    create_dir_all(&nested_dir).unwrap();
    create_dir_all(dest_nested_file.parent().unwrap()).unwrap();
    write(&source_misc_file, "Source misc file").unwrap();
    write(&source_nested_file, "Source nested file").unwrap();
    write(&source_new_nested_file, "I'm not in the tendrils dir").unwrap();
    write(&dest_misc_file, "Existing misc file").unwrap();
    write(&dest_nested_file, "Existing nested file").unwrap();
    write(&dest_extra_nested_file, "I'm not in the source dir").unwrap();

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "SourceDir".to_string(),
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::DirMerge,
    ).unwrap();

    pull_tendril(&given_td_dir, &given, false, force).unwrap();

    let dest_misc_contents = read_to_string(dest_misc_file).unwrap();
    let dest_nested_contents = read_to_string(dest_nested_file).unwrap();
    let dest_new_nested_contents = read_to_string(dest_new_nested_file).unwrap();
    let dest_extra_nested_contents = read_to_string(dest_extra_nested_file).unwrap();
    assert_eq!(dest_misc_contents, "Source misc file");
    assert_eq!(dest_nested_contents, "Source nested file");
    assert_eq!(dest_new_nested_contents, "I'm not in the tendrils dir");
    assert_eq!(dest_extra_nested_contents, "I'm not in the source dir");
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
    setup.make_local_file();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    if dry_run {
        assert!(matches!(actual, Err(TendrilActionError::Skipped)));
        assert!(!setup.td_dir.exists());
    }
    else {
        assert!(matches!(actual, Ok(())));
        assert_eq!(setup.ctrl_file_contents(), "Local file contents");
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn file_tendril_source_is_unchanged(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        setup.parent_dir.clone(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    assert_eq!(setup.local_file_contents(), "Local file contents");
    if dry_run {
        assert!(matches!(actual, Err(TendrilActionError::Skipped)));
    }
    else {
        assert!(matches!(actual, Ok(())));
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
    setup.make_local_file();
    let some_other_ctrl_file= &setup.group_dir.join("other.txt");
    setup.make_group_dir();
    write(some_other_ctrl_file, "Another tendril from the same group").unwrap();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        setup.parent_dir,
        TendrilMode::DirOverwrite,
    ).unwrap();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    if dry_run {
        assert!(matches!(actual, Err(TendrilActionError::Skipped)));
    }
    else {
        assert!(matches!(actual, Ok(())));
    }

    // Check that other tendril is unchanged
    let some_other_ctrl_file_contents = read_to_string(some_other_ctrl_file).unwrap();
    assert_eq!(some_other_ctrl_file_contents, "Another tendril from the same group");
}

#[rstest]
#[case(true)]
#[case(false)]
fn dir_tendril_copies_all_contents_recursively_and_source_is_unchanged(
    #[case] force: bool,
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let given_td_dir = &temp_parent_dir.path().join("TendrilsDir");
    let source= &temp_parent_dir.path().join("SourceDir");
    let nested_dir= &source.join("NestedDir");
    create_dir_all(&nested_dir).unwrap();
    write(&source.join("misc.txt"), "Misc file contents").unwrap();
    write(&nested_dir.join("nested.txt"), "Nested file contents").unwrap();
    let dest_misc_file = given_td_dir
        .join("SomeApp")
        .join("SourceDir")
        .join("misc.txt");
    let dest_nested_file = given_td_dir
        .join("SomeApp")
        .join("SourceDir")
        .join("NestedDir")
        .join("nested.txt");

    let given = ResolvedTendril::new(
        "SomeApp".to_string(),
        "SourceDir".to_string(),
        temp_parent_dir.path().to_path_buf(),
        TendrilMode::DirOverwrite,
    ).unwrap();

    pull_tendril(&given_td_dir, &given, false, force).unwrap();

    let dest_misc_contents = read_to_string(dest_misc_file).unwrap();
    let dest_nested_contents = read_to_string(dest_nested_file).unwrap();
    assert_eq!(dest_misc_contents, "Misc file contents");
    assert_eq!(dest_nested_contents, "Nested file contents");

    // Check that source is unchanged
    let source_misc_contents = read_to_string(source.join("misc.txt")).unwrap();
    let source_nested_contents = read_to_string(nested_dir.join("nested.txt")).unwrap();
    assert_eq!(source_misc_contents, "Misc file contents");
    assert_eq!(source_nested_contents, "Nested file contents");
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
    setup.make_local_file();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        setup.parent_dir.clone(),
        TendrilMode::Link,
    ).unwrap();

    let actual = pull_tendril(&setup.td_dir, &tendril, dry_run, force);

    assert!(matches!(actual, Err(TendrilActionError::ModeMismatch)));
    assert_eq!(&setup.local_file_contents(), "Local file contents");
    assert!(is_empty(&setup.td_dir));
}
