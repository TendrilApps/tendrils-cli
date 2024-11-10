//! Tests that the updater function behaves properly, for additional
//! tests see the similar [`super::batch_tendril_action_tests`] module

use crate::path_ext::UniPath;
use crate::test_utils::{
    get_disposable_dir,
    is_empty,
    set_remotes,
    Setup
};
use crate::{
    batch_tendril_action,
    ActionLog,
    ActionMode,
    FsoType,
    Location,
    TendrilActionError,
    TendrilActionSuccess,
    TendrilBundle,
    TendrilLog,
    TendrilReport,
};
use fs_extra::file::read_to_string;
use rstest::rstest;
use serial_test::serial;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;
use std::rc::Rc;
use tempdir::TempDir;

#[rstest]
fn given_empty_list_returns_empty(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let temp_parent_dir =
        TempDir::new_in(get_disposable_dir(), "ParentDir").unwrap();
    let given_td_repo = temp_parent_dir.path().join("TendrilsRepo").into();
    let mut actual = vec![];
    let updater = |r| actual.push(r);

    batch_tendril_action(updater, mode, &given_td_repo, vec![], dry_run, force);

    assert!(actual.is_empty());
    assert!(is_empty(given_td_repo.inner()))
}

#[rstest]
fn returns_result_after_each_operation(
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_nested_file();
    let mut bundle = setup.file_tendril_bundle();
    bundle.remotes.push(
        setup.remote_nested_file.to_string_lossy().to_string()
    ); // Add another remote

    let expected_success = match dry_run {
        true => Ok(TendrilActionSuccess::NewSkipped),
        false => Ok(TendrilActionSuccess::New),
    };
    let mut call_counter = 0;
    let mut actual = vec![];
    let updater = |r| {
        call_counter += 1;
        if call_counter == 1 {
            assert_eq!(r, TendrilReport {
                orig_tendril: Rc::new(bundle.clone()),
                local: bundle.local.clone(),
                log: Ok(ActionLog::new(
                    Some(FsoType::File),
                    None,
                    setup.remote_file.clone(),
                    expected_success.clone(),
                )),
            });
            if dry_run {
                assert!(!setup.remote_file.exists())
            }
            else {
                assert_eq!(setup.remote_file_contents(), "Local file contents");
            }
            assert!(!setup.remote_dir.exists())
        }
        else if call_counter == 2 {
            assert_eq!(r, TendrilReport {
                orig_tendril: Rc::new(bundle.clone()),
                local: bundle.local.clone(),
                log: Ok(ActionLog::new(
                    Some(FsoType::File),
                    None,
                    setup.remote_nested_file.clone(),
                    expected_success.clone(),
                )),
            });
            if dry_run {
                assert!(!setup.remote_file.exists());
                assert!(!setup.remote_dir.exists());
            }
            else {
                assert_eq!(setup.remote_file_contents(), "Local file contents");
                assert_eq!(
                    setup.remote_nested_file_contents(),
                    "Local file contents" // Note lack of "nested"
                );
            }
        }
        else {
            panic!("Updater was called too many times");
        }

        actual.push(r);
    };

    batch_tendril_action(
        updater,
        ActionMode::Push,
        &UniPath::from(&setup.td_repo),
        vec![bundle.clone()],
        dry_run,
        force,
    );

    assert_eq!(call_counter, 2);
}

#[rstest]
#[case(true)]
#[case(false)]
fn pull_returns_tendril_and_result_for_each_given(
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let temp_grandparent_dir =
        TempDir::new_in(get_disposable_dir(), "ParentDir").unwrap();
    let given_td_repo = temp_grandparent_dir.path().join("TendrilsRepo");
    let given_parent_dir_a = temp_grandparent_dir.path().join("ParentA");
    let given_parent_dir_b = temp_grandparent_dir.path().join("ParentB");
    let remote_app1_file = given_parent_dir_a.join("misc1.txt");
    let remote_app1_dir = given_parent_dir_a.join("App1 Dir");
    let remote_app1_nested_file = remote_app1_dir.join("nested1.txt");
    let remote_app2_file_a = given_parent_dir_a.join("misc2.txt");
    let remote_app2_file_b = given_parent_dir_b.join("misc2.txt");
    let local_app1_file = given_td_repo.join("App1").join("misc1.txt");
    let local_app1_dir = given_td_repo.join("App1").join("App1 Dir");
    let local_app1_nested_file = local_app1_dir.join("nested1.txt");
    let local_app2_file_ab = given_td_repo.join("App2").join("misc2.txt");
    create_dir_all(&given_td_repo).unwrap();
    create_dir_all(&remote_app1_dir).unwrap();
    create_dir_all(&given_parent_dir_a).unwrap();
    create_dir_all(&given_parent_dir_b).unwrap();
    write(&remote_app1_file, "Remote app 1 file contents").unwrap();
    write(&remote_app2_file_a, "Remote app 2 file a contents").unwrap();
    write(&remote_app2_file_b, "Remote app 2 file b contents").unwrap();
    write(&remote_app1_nested_file, "Remote app 1 nested file contents")
        .unwrap();

    let mut given = vec![
        TendrilBundle::new("App1/misc1.txt"),
        TendrilBundle::new("App2/misc2.txt"),
        TendrilBundle::new("App1/App1 Dir"),
        TendrilBundle::new("App3/I don't exist"),
    ];

    set_remotes(&mut given[0], &[given_parent_dir_a.join("misc1.txt")]);
    set_remotes(&mut given[1], &[
        given_parent_dir_a.join("misc2.txt"),
        given_parent_dir_b.join("misc2.txt"),
    ]);
    set_remotes(&mut given[2], &[given_parent_dir_a.join("App1 Dir")]);
    set_remotes(&mut given[3], &[given_parent_dir_a.join("I don't exist")]);

    let expected_success = match dry_run {
        true => Ok(TendrilActionSuccess::NewSkipped),
        false => Ok(TendrilActionSuccess::New),
    };
    let expected = vec![
        TendrilReport {
            orig_tendril: Rc::new(given[0].clone()),
            local: given[0].local.clone(),
            log: Ok(ActionLog::new(
                None,
                Some(FsoType::File),
                remote_app1_file,
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[1].clone()),
            local: given[1].local.clone(),
            log: Ok(ActionLog::new(
                None,
                Some(FsoType::File),
                remote_app2_file_a,
                expected_success.clone(),
            )),
        },
        // The second path should not be considered since this is a pull action
        TendrilReport {
            orig_tendril: Rc::new(given[2].clone()),
            local: given[2].local.clone(),
            log: Ok(ActionLog::new(
                None,
                Some(FsoType::Dir),
                remote_app1_dir,
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            local: given[3].local.clone(),
            log: Ok(ActionLog::new(
                None,
                None,
                given_parent_dir_a.join("I don't exist"),
                Err(TendrilActionError::IoError {
                    kind: std::io::ErrorKind::NotFound,
                    loc: Location::Source,
                }),
            )),
        },
    ];
    let mut actual = vec![];
    let updater = |r| actual.push(r);

    batch_tendril_action(
        updater,
        ActionMode::Pull,
        &UniPath::from(given_td_repo),
        given,
        dry_run,
        force
    );

    assert_eq!(actual, expected);

    if dry_run {
        assert!(!local_app1_file.exists());
        assert!(!local_app1_dir.exists());
        assert!(!local_app2_file_ab.exists());
        assert!(!local_app1_nested_file.exists());
    }
    else {
        let local_app1_file_contents = read_to_string(local_app1_file).unwrap();
        let local_app2_file_contents = read_to_string(local_app2_file_ab).unwrap();
        let local_app1_nested_file_contents =
            read_to_string(local_app1_nested_file).unwrap();

        assert_eq!(local_app1_file_contents, "Remote app 1 file contents");
        assert!(local_app1_dir.exists());
        assert_eq!(local_app2_file_contents, "Remote app 2 file a contents");
        assert_eq!(
            local_app1_nested_file_contents,
            "Remote app 1 nested file contents"
        );
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn push_returns_tendril_and_result_for_each_given(
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let temp_grandparent_dir =
        TempDir::new_in(get_disposable_dir(), "ParentDir").unwrap();
    let given_td_repo = temp_grandparent_dir.path().join("TendrilsRepo");
    let given_parent_dir_a = temp_grandparent_dir.path().join("ParentA");
    let given_parent_dir_b = temp_grandparent_dir.path().join("ParentB");
    let remote_app1_file = given_parent_dir_a.join("misc1.txt");
    let remote_app1_dir = given_parent_dir_a.join("App1 Dir");
    let remote_app1_nested_file = remote_app1_dir.join("nested1.txt");
    let remote_app2_file_a = given_parent_dir_a.join("misc2.txt");
    let remote_app2_file_b = given_parent_dir_b.join("misc2.txt");
    let local_app1_file = given_td_repo.join("App1").join("misc1.txt");
    let local_app1_dir = given_td_repo.join("App1").join("App1 Dir");
    let local_app1_nested_file = local_app1_dir.join("nested1.txt");
    let local_app2_file_ab = given_td_repo.join("App2").join("misc2.txt");
    create_dir_all(given_parent_dir_a.clone()).unwrap();
    create_dir_all(given_parent_dir_b.clone()).unwrap();
    create_dir_all(&local_app1_dir).unwrap();
    create_dir_all(&given_td_repo.join("App2")).unwrap();
    create_dir_all(&given_td_repo.join("App3")).unwrap();
    write(&local_app1_file, "Local app 1 file contents").unwrap();
    write(&local_app2_file_ab, "Local app 2 file contents").unwrap();
    write(&local_app1_nested_file, "Local app 1 nested file contents").unwrap();

    let mut given = vec![
        TendrilBundle::new("App1/misc1.txt"),
        TendrilBundle::new("App2/misc2.txt"),
        TendrilBundle::new("App1/App1 Dir"),
        TendrilBundle::new("App3/I don't exist"),
    ];

    set_remotes(&mut given[0], &[given_parent_dir_a.join("misc1.txt")]);
    set_remotes(&mut given[1], &[
        given_parent_dir_a.join("misc2.txt"),
        given_parent_dir_b.join("misc2.txt"),
    ]);
    set_remotes(&mut given[2], &[given_parent_dir_a.join("App1 Dir")]);
    set_remotes(&mut given[3], &[given_parent_dir_a.join("I don't exist")]);

    let expected_success = match dry_run {
        true => Ok(TendrilActionSuccess::NewSkipped),
        false => Ok(TendrilActionSuccess::New),
    };
    let expected = vec![
        TendrilReport {
            orig_tendril: Rc::new(given[0].clone()),
            local: given[0].local.clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app1_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[1].clone()),
            local: given[1].local.clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app2_file_a.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[1].clone()),
            local: given[1].local.clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app2_file_b.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[2].clone()),
            local: given[2].local.clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::Dir),
                None,
                remote_app1_dir.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            local: given[3].local.clone(),
            log: Ok(ActionLog::new(
                None,
                None,
                given_parent_dir_a.join("I don't exist"),
                Err(TendrilActionError::IoError {
                    kind: std::io::ErrorKind::NotFound,
                    loc: Location::Source,
                }),
            )),
        },
    ];
    let mut actual = vec![];
    let updater = |r| actual.push(r);

    batch_tendril_action(
        updater,
        ActionMode::Push,
        &UniPath::from(given_td_repo),
        given,
        dry_run,
        force,
    );

    assert_eq!(actual, expected);

    if dry_run {
        assert!(!remote_app1_file.exists());
        assert!(!remote_app2_file_a.exists());
        assert!(!remote_app2_file_b.exists());
        assert!(!remote_app1_nested_file.exists());
    }
    else {
        let remote_app1_file_contents =
            read_to_string(&remote_app1_file).unwrap();
        let remote_app2_file_a_contents =
            read_to_string(&remote_app2_file_a).unwrap();
        let remote_app2_file_b_contents =
            read_to_string(&remote_app2_file_b).unwrap();
        let remote_app1_nested_file_contents =
            read_to_string(&remote_app1_nested_file).unwrap();

        assert_eq!(remote_app1_file_contents, "Local app 1 file contents");
        assert_eq!(remote_app2_file_a_contents, "Local app 2 file contents");
        assert_eq!(remote_app2_file_b_contents, "Local app 2 file contents");
        assert_eq!(
            remote_app1_nested_file_contents,
            "Local app 1 nested file contents"
        );
        assert!(!remote_app1_file.is_symlink());
        assert!(!remote_app2_file_a.is_symlink());
        assert!(!remote_app2_file_b.is_symlink());
        assert!(!remote_app1_dir.is_symlink());
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn link_returns_tendril_and_result_for_each_given(
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let temp_grandparent_dir =
        TempDir::new_in(get_disposable_dir(), "ParentDir").unwrap();
    let given_td_repo = temp_grandparent_dir.path().join("TendrilsRepo");
    let given_parent_dir_a = temp_grandparent_dir.path().join("ParentA");
    let given_parent_dir_b = temp_grandparent_dir.path().join("ParentB");
    let remote_app1_file = given_parent_dir_a.join("misc1.txt");
    let remote_app1_dir = given_parent_dir_a.join("App1 Dir");
    let remote_app1_nested_file = remote_app1_dir.join("nested1.txt");
    let remote_app2_file_a = given_parent_dir_a.join("misc2.txt");
    let remote_app2_file_b = given_parent_dir_b.join("misc2.txt");
    let local_app1_file = given_td_repo.join("App1").join("misc1.txt");
    let local_app1_dir = given_td_repo.join("App1").join("App1 Dir");
    let local_app1_nested_file = local_app1_dir.join("nested1.txt");
    let local_app2_file_ab = given_td_repo.join("App2").join("misc2.txt");
    create_dir_all(given_parent_dir_a.clone()).unwrap();
    create_dir_all(given_parent_dir_b.clone()).unwrap();
    create_dir_all(&local_app1_dir).unwrap();
    create_dir_all(&given_td_repo.join("App2")).unwrap();
    create_dir_all(&given_td_repo.join("App3")).unwrap();
    write(&local_app1_file, "Local app 1 file contents").unwrap();
    write(&local_app2_file_ab, "Local app 2 file contents").unwrap();
    write(&local_app1_nested_file, "Local app 1 nested file contents").unwrap();

    let mut given = vec![
        TendrilBundle::new("App1/misc1.txt"),
        TendrilBundle::new("App2/misc2.txt"),
        TendrilBundle::new("App1/App1 Dir"),
        TendrilBundle::new("App3/I don't exist"),
    ];

    set_remotes(&mut given[0], &[given_parent_dir_a.join("misc1.txt")]);
    set_remotes(&mut given[1], &[
        given_parent_dir_a.join("misc2.txt"),
        given_parent_dir_b.join("misc2.txt"),
    ]);
    set_remotes(&mut given[2], &[given_parent_dir_a.join("App1 Dir")]);
    set_remotes(&mut given[3], &[given_parent_dir_a.join("I don't exist")]);

    given[0].link = true;
    given[1].link = true;
    given[2].link = true;
    given[3].link = true;

    let expected_success = match dry_run {
        true => Ok(TendrilActionSuccess::NewSkipped),
        false => Ok(TendrilActionSuccess::New),
    };
    let expected = vec![
        TendrilReport {
            orig_tendril: Rc::new(given[0].clone()),
            local: given[0].local.clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app1_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[1].clone()),
            local: given[1].local.clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app2_file_a.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[1].clone()),
            local: given[1].local.clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app2_file_b.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[2].clone()),
            local: given[2].local.clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::Dir),
                None,
                remote_app1_dir.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            local: given[3].local.clone(),
            log: Ok(ActionLog::new(
                None,
                None,
                given_parent_dir_a.join("I don't exist"),
                Err(TendrilActionError::IoError {
                    kind: std::io::ErrorKind::NotFound,
                    loc: Location::Source,
                }),
            )),
        },
    ];
    let mut actual = vec![];
    let updater = |r| actual.push(r);

    batch_tendril_action(
        updater,
        ActionMode::Link,
        &UniPath::from(given_td_repo),
        given,
        dry_run,
        force,
    );

    assert_eq!(actual, expected);

    if dry_run {
        assert!(!remote_app1_file.exists());
        assert!(!remote_app2_file_a.exists());
        assert!(!remote_app2_file_b.exists());
        assert!(!remote_app1_nested_file.exists());
    }
    else {
        let remote_app1_file_contents =
            read_to_string(&remote_app1_file).unwrap();
        let remote_app2_file_a_contents =
            read_to_string(&remote_app2_file_a).unwrap();
        let remote_app2_file_b_contents =
            read_to_string(&remote_app2_file_b).unwrap();
        let remote_app1_nested_file_contents =
            read_to_string(&remote_app1_nested_file).unwrap();

        assert_eq!(remote_app1_file_contents, "Local app 1 file contents");
        assert_eq!(remote_app2_file_a_contents, "Local app 2 file contents");
        assert_eq!(remote_app2_file_b_contents, "Local app 2 file contents");
        assert_eq!(
            remote_app1_nested_file_contents,
            "Local app 1 nested file contents"
        );
        assert!(remote_app1_file.is_symlink());
        assert!(remote_app2_file_a.is_symlink());
        assert!(remote_app2_file_b.is_symlink());
        assert!(remote_app1_dir.is_symlink());
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn out_returns_tendril_and_result_for_each_given_link_or_push_style(
    #[case] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let temp_grandparent_dir =
        TempDir::new_in(get_disposable_dir(), "ParentDir").unwrap();
    let given_td_repo = temp_grandparent_dir.path().join("TendrilsRepo");
    let given_parent_dir_a = temp_grandparent_dir.path().join("ParentA");
    let given_parent_dir_b = temp_grandparent_dir.path().join("ParentB");
    let remote_app1_file = given_parent_dir_a.join("misc1.txt");
    let remote_app1_dir = given_parent_dir_a.join("App1 Dir");
    let remote_app1_nested_file = remote_app1_dir.join("nested1.txt");
    let remote_app2_file_a = given_parent_dir_a.join("misc2.txt");
    let remote_app2_file_b = given_parent_dir_b.join("misc2.txt");
    let local_app1_file = given_td_repo.join("App1").join("misc1.txt");
    let local_app1_dir = given_td_repo.join("App1").join("App1 Dir");
    let local_app1_nested_file = local_app1_dir.join("nested1.txt");
    let local_app2_file_ab = given_td_repo.join("App2").join("misc2.txt");
    create_dir_all(given_parent_dir_a.clone()).unwrap();
    create_dir_all(given_parent_dir_b.clone()).unwrap();
    create_dir_all(&local_app1_dir).unwrap();
    create_dir_all(&given_td_repo.join("App2")).unwrap();
    create_dir_all(&given_td_repo.join("App3")).unwrap();
    write(&local_app1_file, "Local app 1 file contents").unwrap();
    write(&local_app2_file_ab, "Local app 2 file contents").unwrap();
    write(&local_app1_nested_file, "Local app 1 nested file contents").unwrap();

    let mut given = vec![
        TendrilBundle::new("App1/misc1.txt"),
        TendrilBundle::new("App2/misc2.txt"),
        TendrilBundle::new("App1/App1 Dir"),
        TendrilBundle::new("App3/I don't exist"),
    ];

    set_remotes(&mut given[0], &[given_parent_dir_a.join("misc1.txt")]);
    set_remotes(&mut given[1], &[
        given_parent_dir_a.join("misc2.txt"),
        given_parent_dir_b.join("misc2.txt"),
    ]);
    set_remotes(&mut given[2], &[given_parent_dir_a.join("App1 Dir")]);
    set_remotes(&mut given[3], &[given_parent_dir_a.join("I don't exist")]);

    given[0].link = true;
    given[1].link = false;
    given[2].link = true;
    given[3].link = false;

    let expected_success = match dry_run {
        true => Ok(TendrilActionSuccess::NewSkipped),
        false => Ok(TendrilActionSuccess::New),
    };
    let expected = vec![
        TendrilReport {
            orig_tendril: Rc::new(given[0].clone()),
            local: given[0].local.clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app1_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[1].clone()),
            local: given[1].local.clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app2_file_a.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[1].clone()),
            local: given[1].local.clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app2_file_b.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[2].clone()),
            local: given[2].local.clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::Dir),
                None,
                remote_app1_dir.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            local: given[3].local.clone(),
            log: Ok(ActionLog::new(
                None,
                None,
                given_parent_dir_a.join("I don't exist"),
                Err(TendrilActionError::IoError {
                    kind: std::io::ErrorKind::NotFound,
                    loc: Location::Source,
                }),
            )),
        },
    ];
    let mut actual = vec![];
    let updater = |r| actual.push(r);

    batch_tendril_action(
        updater,
        ActionMode::Out,
        &UniPath::from(given_td_repo),
        given,
        dry_run,
        force,
    );

    assert_eq!(actual, expected);

    if dry_run {
        assert!(!remote_app1_file.exists());
        assert!(!remote_app2_file_a.exists());
        assert!(!remote_app2_file_b.exists());
        assert!(!remote_app1_nested_file.exists());
    }
    else {
        let remote_app1_file_contents =
            read_to_string(&remote_app1_file).unwrap();
        let remote_app2_file_a_contents =
            read_to_string(&remote_app2_file_a).unwrap();
        let remote_app2_file_b_contents =
            read_to_string(&remote_app2_file_b).unwrap();
        let remote_app1_nested_file_contents =
            read_to_string(&remote_app1_nested_file).unwrap();

        assert_eq!(remote_app1_file_contents, "Local app 1 file contents");
        assert_eq!(remote_app2_file_a_contents, "Local app 2 file contents");
        assert_eq!(remote_app2_file_b_contents, "Local app 2 file contents");
        assert_eq!(
            remote_app1_nested_file_contents,
            "Local app 1 nested file contents"
        );
        assert!(remote_app1_file.is_symlink());
        assert!(!remote_app2_file_a.is_symlink());
        assert!(!remote_app2_file_b.is_symlink());
        assert!(remote_app1_dir.is_symlink());
    }
}

#[rstest]
#[serial("mut-env-var-testing")]
fn remote_path_vars_are_resolved(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_td_repo_dir();
    let mut tendril = setup.file_tendril_bundle();
    tendril.link = mode == ActionMode::Link;
    set_remotes(&mut tendril, &[PathBuf::from("~/I_do_not_exist/<var>/misc.txt")]);
    let tendrils = vec![tendril.clone()];
    std::env::set_var("HOME", "My/Home");
    std::env::set_var("var", "value");

    use std::path::MAIN_SEPARATOR as SEP;
    let expected_resolved_path = format!(
        "{SEP}My{SEP}Home{SEP}I_do_not_exist{SEP}value{SEP}misc.txt"
    );
    let expected = vec![TendrilReport {
        orig_tendril: Rc::new(tendril),
        local: "SomeApp/misc.txt".to_string(),
        log: Ok(ActionLog::new(
            None,
            None,
            PathBuf::from(expected_resolved_path.clone()),
            Err(TendrilActionError::IoError {
                kind: std::io::ErrorKind::NotFound,
                loc: Location::Source,
            }),
        )),
    }];

    let mut actual = vec![];
    let updater = |r| actual.push(r);

    batch_tendril_action(
        updater,
        mode,
        &UniPath::from(&setup.td_repo),
        tendrils,
        dry_run,
        force,
    );

    let actual_result_path = &actual[0].log.as_ref().unwrap().resolved_path();

    let actual_resolved_path_str = actual_result_path.to_string_lossy();
    assert_eq!(actual_resolved_path_str.into_owned(), expected_resolved_path);
    assert_eq!(actual, expected);
}

// TODO: Test when the second tendril is a parent/child to the first tendril
