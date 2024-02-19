use crate::{link_tendril, symlink};
use crate::enums::{TendrilActionError, TendrilActionSuccess};
use crate::resolved_tendril::{ResolvedTendril, TendrilMode};
use crate::test_utils::{is_empty, Setup};
use rstest::rstest;
use std::path::PathBuf;
use std::fs::create_dir_all;

#[rstest]
fn tendril_is_not_link_mode_returns_mode_mismatch_error(
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

#[rstest]
#[case(true)]
#[case(false)]
fn remote_parent_doesnt_exist_returns_io_error_not_found(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_ctrl_file();

    let tendril = ResolvedTendril::new(
        "SomeApp".to_string(),
        "misc.txt".to_string(),
        PathBuf::from("SomePathThatDoesNotExist"),
        TendrilMode::Link,
    ).unwrap();

    let actual = link_tendril(&setup.td_dir, &tendril, dry_run, force);

    match actual {
        Err(TendrilActionError::IoError(e)) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        _ => panic!("Actual error: {:?}", actual),
    }
    assert_eq!(setup.ctrl_file_contents(), "Controlled file contents");
    assert_eq!(setup.parent_dir.read_dir().iter().count(), 1);
}

#[rstest]
#[case(true)]
#[case(false)]
fn ctrl_doesnt_exist_returns_io_error_not_found(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_parent_dir();

    let mut tendril = setup.resolved_file_tendril();
    tendril.mode = TendrilMode::Link;

    let actual = link_tendril(&setup.td_dir, &tendril, dry_run, force);

    match actual {
        Err(TendrilActionError::IoError(e)) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        _ => panic!("Actual error: {:?}", actual),
    }
    assert!(!setup.remote_file.exists());
    assert!(is_empty(&setup.parent_dir));
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
    setup.make_ctrl_file();
    setup.make_ctrl_nested_file();

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
        assert_eq!(setup.remote_file_contents(), "Controlled file contents");
        assert_eq!(
            setup.remote_nested_file_contents(),
            "Controlled nested file contents"
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
fn ctrl_is_symlink_returns_type_mismatch_error_unless_forced(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_target_file();
    setup.make_target_nested_file();
    create_dir_all(&setup.group_dir).unwrap();
    symlink(&setup.ctrl_file, &setup.target_file, false, false).unwrap();
    symlink(&setup.ctrl_dir, &setup.target_dir, false, false).unwrap();

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

    assert_eq!(setup.ctrl_file_contents(), "Target file contents");
    assert_eq!(
        setup.ctrl_nested_file_contents(),
        "Target nested file contents"
    );
    assert!(setup.ctrl_file.is_symlink());
    assert!(setup.ctrl_dir.is_symlink());
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

#[rstest]
#[case(true)]
#[case(false)]
fn remote_doesnt_exist_but_parent_does_symlink_is_created(#[case] force: bool) {
    let setup = Setup::new();
    setup.make_ctrl_file();
    setup.make_ctrl_nested_file();
    assert!(!setup.remote_file.exists());
    assert!(!setup.remote_dir.exists());

    let mut file_tendril = setup.resolved_file_tendril();
    file_tendril.mode = TendrilMode::Link;

    let mut dir_tendril = setup.resolved_dir_tendril();
    dir_tendril.mode = TendrilMode::Link;

    link_tendril(&setup.td_dir, &file_tendril, false, force).unwrap();
    link_tendril(&setup.td_dir, &dir_tendril, false, force).unwrap();

    assert!(setup.remote_file.is_symlink());
    assert!(setup.remote_dir.is_symlink());
    assert_eq!(setup.remote_file_contents(), "Controlled file contents");
    assert_eq!(
        setup.remote_nested_file_contents(),
        "Controlled nested file contents"
    );
}

#[rstest]
#[case(true)]
#[case(false)]
fn remote_doesnt_exist_but_parent_does_symlink_not_created_in_dry_run(
    #[case] force: bool
) {
    let setup = Setup::new();
    setup.make_ctrl_file();
    setup.make_ctrl_nested_file();

    let mut file_tendril = setup.resolved_file_tendril();
    file_tendril.mode = TendrilMode::Link;

    let mut dir_tendril = setup.resolved_dir_tendril();
    dir_tendril.mode = TendrilMode::Link;

    let file_actual = link_tendril(&setup.td_dir, &file_tendril, true, force);
    let dir_actual = link_tendril(&setup.td_dir, &dir_tendril, true, force);

    assert!(matches!(file_actual, Ok(TendrilActionSuccess::Skipped)));
    assert!(matches!(dir_actual, Ok(TendrilActionSuccess::Skipped)));
    assert!(!setup.remote_file.exists());
    assert!(!setup.remote_dir.exists());
}

#[rstest]
#[case(true)]
#[case(false)]
fn existing_symlinks_at_remote_are_overwritten(#[case] force: bool) {
    let setup = Setup::new();
    setup.make_ctrl_file();
    setup.make_ctrl_nested_file();
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
    assert_eq!(setup.remote_file_contents(), "Controlled file contents");
    assert_eq!(
        setup.remote_nested_file_contents(),
        "Controlled nested file contents"
    );

    assert!(setup.remote_file.is_symlink());
    assert!(setup.remote_dir.is_symlink());
    assert_eq!(setup.remote_file_contents(), "Controlled file contents");
    assert_eq!(setup.remote_nested_file_contents(), "Controlled nested file contents");
}

#[rstest]
#[case(true)]
#[case(false)]
fn existing_symlinks_at_remote_are_unmodified_in_dry_run(#[case] force: bool) {
    let setup = Setup::new();
    setup.make_ctrl_file();
    setup.make_ctrl_nested_file();
    setup.make_target_file();
    setup.make_target_nested_file();
    symlink(&setup.remote_file, &setup.target_file, false, true).unwrap();
    symlink(&setup.remote_dir, &setup.target_dir, false, true).unwrap();

    let mut file_tendril = setup.resolved_file_tendril();
    file_tendril.mode = TendrilMode::Link;

    let mut dir_tendril = setup.resolved_dir_tendril();
    dir_tendril.mode = TendrilMode::Link;

    let file_actual = link_tendril(&setup.td_dir, &file_tendril, true, force);
    let dir_actual = link_tendril(&setup.td_dir, &dir_tendril, true, force);

    assert!(matches!(file_actual, Ok(TendrilActionSuccess::Skipped)));
    assert!(matches!(dir_actual, Ok(TendrilActionSuccess::Skipped)));
    assert!(setup.remote_file.is_symlink());
    assert!(setup.remote_dir.is_symlink());
    assert_eq!(setup.remote_file_contents(), "Target file contents");
    assert_eq!(setup.remote_nested_file_contents(), "Target nested file contents");
}
