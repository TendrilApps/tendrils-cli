//! Tests that the updater function behaves properly, for additional
//! tests see the similar [`super::batch_tendril_action_tests`] module

use crate::test_utils::{
    get_disposable_dir,
    is_empty,
    set_parents,
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
    let given_td_repo = temp_parent_dir.path().join("TendrilsRepo");
    let mut actual = vec![];
    let updater = |r| actual.push(r);

    batch_tendril_action(updater, mode, &given_td_repo, vec![], dry_run, force);

    assert!(actual.is_empty());
    assert!(is_empty(&given_td_repo))
}

#[rstest]
fn returns_result_after_each_operation(
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_remote_nested_file();
    setup.make_td_repo_dir();
    let mut bundle = setup.file_tendril_bundle();
    bundle.names.push("misc".to_string()); // Add the folder

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
                name: bundle.names[0].clone(),
                log: Ok(ActionLog::new(
                    None,
                    Some(FsoType::File),
                    setup.remote_file.clone(),
                    expected_success.clone(),
                )),
            });
            if dry_run {
                assert!(!setup.local_file.exists())
            }
            else {
                assert_eq!(setup.local_file_contents(), "Remote file contents");
            }
            assert!(!setup.local_dir.exists())
        }
        else if call_counter == 2 {
            assert_eq!(r, TendrilReport {
                orig_tendril: Rc::new(bundle.clone()),
                name: bundle.names[1].clone(),
                log: Ok(ActionLog::new(
                    None,
                    Some(FsoType::Dir),
                    setup.remote_dir.clone(),
                    expected_success.clone(),
                )),
            });
            if dry_run {
                assert!(!setup.local_file.exists());
                assert!(!setup.local_dir.exists());
            }
            else {
                assert_eq!(setup.local_file_contents(), "Remote file contents");
                assert_eq!(
                    setup.local_nested_file_contents(),
                    "Remote nested file contents"
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
        ActionMode::Pull,
        &setup.td_repo,
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
    let remote_app2_file = given_parent_dir_a.join("misc2.txt");
    let remote_app3_fileb_pa = given_parent_dir_a.join("misc3.txt");
    let local_app1_file = given_td_repo.join("App1").join("misc1.txt");
    let local_app1_dir = given_td_repo.join("App1").join("App1 Dir");
    let local_app1_nested_file = local_app1_dir.join("nested1.txt");
    let local_app2_file = given_td_repo.join("App2").join("misc2.txt");
    let local_app3_file_b = given_td_repo.join("App3").join("misc3.txt");
    create_dir_all(&given_td_repo).unwrap();
    create_dir_all(&remote_app1_dir).unwrap();
    write(&remote_app1_file, "Remote app 1 file contents").unwrap();
    write(&remote_app2_file, "Remote app 2 file contents").unwrap();
    write(&remote_app3_fileb_pa, "Remote app 3 file b parent a contents")
        .unwrap();
    write(&remote_app1_nested_file, "Remote app 1 nested file contents")
        .unwrap();

    let mut given = vec![
        TendrilBundle::new("App1", vec!["misc1.txt"]),
        TendrilBundle::new("App2", vec!["misc2.txt"]),
        TendrilBundle::new("App1", vec!["App1 Dir"]),
        TendrilBundle::new("App3", vec!["I don't exist", "misc3.txt"]),
    ];

    set_parents(&mut given[0], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[1], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[2], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[3], &[
        given_parent_dir_a.clone(),
        given_parent_dir_b.clone(),
    ]);

    let expected_success = match dry_run {
        true => Ok(TendrilActionSuccess::NewSkipped),
        false => Ok(TendrilActionSuccess::New),
    };
    let expected = vec![
        TendrilReport {
            orig_tendril: Rc::new(given[0].clone()),
            name: given[0].names[0].clone(),
            log: Ok(ActionLog::new(
                None,
                Some(FsoType::File),
                remote_app1_file,
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[1].clone()),
            name: given[1].names[0].clone(),
            log: Ok(ActionLog::new(
                None,
                Some(FsoType::File),
                remote_app2_file,
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[2].clone()),
            name: given[2].names[0].clone(),
            log: Ok(ActionLog::new(
                None,
                Some(FsoType::Dir),
                remote_app1_dir,
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            name: given[3].names[0].clone(),
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
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            name: given[3].names[1].clone(),
            log: Ok(ActionLog::new(
                None,
                Some(FsoType::File),
                remote_app3_fileb_pa,
                expected_success,
            )),
        },
        // The second path should not be considered since this is a pull action
    ];
    let mut actual = vec![];
    let updater = |r| actual.push(r);

    batch_tendril_action(
        updater,
        ActionMode::Pull,
        &given_td_repo,
        given,
        dry_run,
        force
    );

    assert_eq!(actual, expected);

    if dry_run {
        assert!(!local_app1_file.exists());
        assert!(!local_app1_dir.exists());
        assert!(!local_app2_file.exists());
        assert!(!local_app3_file_b.exists());
        assert!(!local_app1_nested_file.exists());
    }
    else {
        let local_app1_file_contents = read_to_string(local_app1_file).unwrap();
        let local_app2_file_contents = read_to_string(local_app2_file).unwrap();
        let local_app3_fileb_contents =
            read_to_string(local_app3_file_b).unwrap();
        let local_app1_nested_file_contents =
            read_to_string(local_app1_nested_file).unwrap();

        assert_eq!(local_app1_file_contents, "Remote app 1 file contents");
        assert!(local_app1_dir.exists());
        assert_eq!(local_app2_file_contents, "Remote app 2 file contents");
        assert_eq!(
            local_app3_fileb_contents,
            "Remote app 3 file b parent a contents"
        );
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
    let remote_app2_file = given_parent_dir_a.join("misc2.txt");
    let remote_app3_fileb_pa = given_parent_dir_a.join("misc3.txt");
    let remote_app3_fileb_pb = given_parent_dir_b.join("misc3.txt");
    let local_app1_file = given_td_repo.join("App1").join("misc1.txt");
    let local_app1_dir = given_td_repo.join("App1").join("App1 Dir");
    let local_nested_app1_file = local_app1_dir.join("nested1.txt");
    let local_app2_file = given_td_repo.join("App2").join("misc2.txt");
    let local_app3_file_b = given_td_repo.join("App3").join("misc3.txt");
    create_dir_all(given_parent_dir_a.clone()).unwrap();
    create_dir_all(given_parent_dir_b.clone()).unwrap();
    create_dir_all(&local_app1_dir).unwrap();
    create_dir_all(&given_td_repo.join("App2")).unwrap();
    create_dir_all(&given_td_repo.join("App3")).unwrap();
    write(&local_app1_file, "Local app 1 file contents").unwrap();
    write(&local_app2_file, "Local app 2 file contents").unwrap();
    write(&local_app3_file_b, "Local app 3 file b contents").unwrap();
    write(&local_nested_app1_file, "Local app 1 nested file contents").unwrap();

    let mut given = vec![
        TendrilBundle::new("App1", vec!["misc1.txt"]),
        TendrilBundle::new("App2", vec!["misc2.txt"]),
        TendrilBundle::new("App1", vec!["App1 Dir"]),
        TendrilBundle::new("App3", vec!["I don't exist", "misc3.txt"]),
    ];

    set_parents(&mut given[0], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[1], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[2], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[3], &[
        given_parent_dir_a.clone(),
        given_parent_dir_b.clone(),
    ]);

    let expected_success = match dry_run {
        true => Ok(TendrilActionSuccess::NewSkipped),
        false => Ok(TendrilActionSuccess::New),
    };
    let expected = vec![
        TendrilReport {
            orig_tendril: Rc::new(given[0].clone()),
            name: given[0].names[0].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app1_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[1].clone()),
            name: given[1].names[0].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app2_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[2].clone()),
            name: given[2].names[0].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::Dir),
                None,
                remote_app1_dir.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            name: given[3].names[0].clone(),
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
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            name: given[3].names[0].clone(),
            log: Ok(ActionLog::new(
                None,
                None,
                given_parent_dir_b.join("I don't exist"),
                Err(TendrilActionError::IoError {
                    kind: std::io::ErrorKind::NotFound,
                    loc: Location::Source,
                }),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            name: given[3].names[1].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app3_fileb_pa.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            name: given[3].names[1].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app3_fileb_pb.clone(),
                expected_success,
            )),
        },
    ];
    let mut actual = vec![];
    let updater = |r| actual.push(r);

    batch_tendril_action(
        updater,
        ActionMode::Push,
        &given_td_repo,
        given,
        dry_run,
        force,
    );

    assert_eq!(actual, expected);

    if dry_run {
        assert!(!remote_app1_file.exists());
        assert!(!remote_app2_file.exists());
        assert!(!remote_app3_fileb_pa.exists());
        assert!(!remote_app3_fileb_pb.exists());
        assert!(!remote_app1_nested_file.exists());
    }
    else {
        let remote_app1_file_contents =
            read_to_string(&remote_app1_file).unwrap();
        let remote_app2_file_contents =
            read_to_string(&remote_app2_file).unwrap();
        let remote_app3_fileb_pa_contents =
            read_to_string(&remote_app3_fileb_pa).unwrap();
        let remote_app3_fileb_pb_contents =
            read_to_string(&remote_app3_fileb_pb).unwrap();
        let remote_app1_nested_file_contents =
            read_to_string(&remote_app1_nested_file).unwrap();

        assert_eq!(remote_app1_file_contents, "Local app 1 file contents");
        assert_eq!(remote_app2_file_contents, "Local app 2 file contents");
        assert_eq!(
            remote_app3_fileb_pa_contents,
            "Local app 3 file b contents"
        );
        assert_eq!(
            remote_app3_fileb_pb_contents,
            "Local app 3 file b contents"
        );
        assert_eq!(
            remote_app1_nested_file_contents,
            "Local app 1 nested file contents"
        );
        assert!(!remote_app1_file.is_symlink());
        assert!(!remote_app2_file.is_symlink());
        assert!(!remote_app3_fileb_pa.is_symlink());
        assert!(!remote_app3_fileb_pb.is_symlink());
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
    let remote_app2_file = given_parent_dir_a.join("misc2.txt");
    let remote_app3_fileb_pa = given_parent_dir_a.join("misc3.txt");
    let remote_app3_fileb_pb = given_parent_dir_b.join("misc3.txt");
    let local_app1_file = given_td_repo.join("App1").join("misc1.txt");
    let local_app1_dir = given_td_repo.join("App1").join("App1 Dir");
    let local_nested_app1_file = local_app1_dir.join("nested1.txt");
    let local_app2_file = given_td_repo.join("App2").join("misc2.txt");
    let local_app3_file_b = given_td_repo.join("App3").join("misc3.txt");
    create_dir_all(given_parent_dir_a.clone()).unwrap();
    create_dir_all(given_parent_dir_b.clone()).unwrap();
    create_dir_all(&local_app1_dir).unwrap();
    create_dir_all(&given_td_repo.join("App2")).unwrap();
    create_dir_all(&given_td_repo.join("App3")).unwrap();
    write(&local_app1_file, "Local app 1 file contents").unwrap();
    write(&local_app2_file, "Local app 2 file contents").unwrap();
    write(&local_app3_file_b, "Local app 3 file b contents").unwrap();
    write(&local_nested_app1_file, "Local app 1 nested file contents").unwrap();

    let mut given = vec![
        TendrilBundle::new("App1", vec!["misc1.txt"]),
        TendrilBundle::new("App2", vec!["misc2.txt"]),
        TendrilBundle::new("App1", vec!["App1 Dir"]),
        TendrilBundle::new("App3", vec!["I don't exist", "misc3.txt"]),
    ];

    given[0].link = true;
    given[1].link = true;
    given[2].link = true;
    given[3].link = true;

    set_parents(&mut given[0], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[1], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[2], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[3], &[
        given_parent_dir_a.clone(),
        given_parent_dir_b.clone(),
    ]);

    let expected_success = match dry_run {
        true => Ok(TendrilActionSuccess::NewSkipped),
        false => Ok(TendrilActionSuccess::New),
    };
    let expected = vec![
        TendrilReport {
            orig_tendril: Rc::new(given[0].clone()),
            name: given[0].names[0].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app1_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[1].clone()),
            name: given[1].names[0].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app2_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[2].clone()),
            name: given[2].names[0].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::Dir),
                None,
                remote_app1_dir.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            name: given[3].names[0].clone(),
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
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            name: given[3].names[0].clone(),
            log: Ok(ActionLog::new(
                None,
                None,
                given_parent_dir_b.join("I don't exist"),
                Err(TendrilActionError::IoError {
                    kind: std::io::ErrorKind::NotFound,
                    loc: Location::Source,
                }),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            name: given[3].names[1].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app3_fileb_pa.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            name: given[3].names[1].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app3_fileb_pb.clone(),
                expected_success,
            )),
        },
    ];
    let mut actual = vec![];
    let updater = |r| actual.push(r);

    batch_tendril_action(
        updater,
        ActionMode::Link,
        &given_td_repo,
        given,
        dry_run,
        force,
    );

    assert_eq!(actual, expected);

    if dry_run {
        assert!(!remote_app1_file.exists());
        assert!(!remote_app2_file.exists());
        assert!(!remote_app3_fileb_pa.exists());
        assert!(!remote_app3_fileb_pb.exists());
        assert!(!remote_app1_nested_file.exists());
    }
    else {
        let remote_app1_file_contents =
            read_to_string(&remote_app1_file).unwrap();
        let remote_app2_file_contents =
            read_to_string(&remote_app2_file).unwrap();
        let remote_app3_fileb_pa_contents =
            read_to_string(&remote_app3_fileb_pa).unwrap();
        let remote_app3_fileb_pb_contents =
            read_to_string(&remote_app3_fileb_pb).unwrap();
        let remote_app1_nested_file_contents =
            read_to_string(&remote_app1_nested_file).unwrap();

        assert_eq!(remote_app1_file_contents, "Local app 1 file contents");
        assert_eq!(remote_app2_file_contents, "Local app 2 file contents");
        assert_eq!(
            remote_app3_fileb_pa_contents,
            "Local app 3 file b contents"
        );
        assert_eq!(
            remote_app3_fileb_pb_contents,
            "Local app 3 file b contents"
        );
        assert_eq!(
            remote_app1_nested_file_contents,
            "Local app 1 nested file contents"
        );
        assert!(remote_app1_file.is_symlink());
        assert!(remote_app2_file.is_symlink());
        assert!(remote_app3_fileb_pa.is_symlink());
        assert!(remote_app3_fileb_pb.is_symlink());
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
    let remote_app2_file = given_parent_dir_a.join("misc2.txt");
    let remote_app3_fileb_pa = given_parent_dir_a.join("misc3.txt");
    let remote_app3_fileb_pb = given_parent_dir_b.join("misc3.txt");
    let local_app1_file = given_td_repo.join("App1").join("misc1.txt");
    let local_app1_dir = given_td_repo.join("App1").join("App1 Dir");
    let local_nested_app1_file = local_app1_dir.join("nested1.txt");
    let local_app2_file = given_td_repo.join("App2").join("misc2.txt");
    let local_app3_file_b = given_td_repo.join("App3").join("misc3.txt");
    create_dir_all(given_parent_dir_a.clone()).unwrap();
    create_dir_all(given_parent_dir_b.clone()).unwrap();
    create_dir_all(&local_app1_dir).unwrap();
    create_dir_all(&given_td_repo.join("App2")).unwrap();
    create_dir_all(&given_td_repo.join("App3")).unwrap();
    write(&local_app1_file, "Local app 1 file contents").unwrap();
    write(&local_app2_file, "Local app 2 file contents").unwrap();
    write(&local_app3_file_b, "Local app 3 file b contents").unwrap();
    write(&local_nested_app1_file, "Local app 1 nested file contents").unwrap();

    let mut given = vec![
        TendrilBundle::new("App1", vec!["misc1.txt"]),
        TendrilBundle::new("App2", vec!["misc2.txt"]),
        TendrilBundle::new("App1", vec!["App1 Dir"]),
        TendrilBundle::new("App3", vec!["I don't exist", "misc3.txt"]),
    ];

    given[0].link = true;
    given[1].link = false;
    given[2].link = true;
    given[3].link = false;

    set_parents(&mut given[0], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[1], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[2], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[3], &[
        given_parent_dir_a.clone(),
        given_parent_dir_b.clone(),
    ]);

    let expected_success = match dry_run {
        true => Ok(TendrilActionSuccess::NewSkipped),
        false => Ok(TendrilActionSuccess::New),
    };
    let expected = vec![
        TendrilReport {
            orig_tendril: Rc::new(given[0].clone()),
            name: given[0].names[0].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app1_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[1].clone()),
            name: given[1].names[0].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app2_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[2].clone()),
            name: given[2].names[0].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::Dir),
                None,
                remote_app1_dir.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            name: given[3].names[0].clone(),
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
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            name: given[3].names[0].clone(),
            log: Ok(ActionLog::new(
                None,
                None,
                given_parent_dir_b.join("I don't exist"),
                Err(TendrilActionError::IoError {
                    kind: std::io::ErrorKind::NotFound,
                    loc: Location::Source,
                }),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            name: given[3].names[1].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app3_fileb_pa.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: Rc::new(given[3].clone()),
            name: given[3].names[1].clone(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app3_fileb_pb.clone(),
                expected_success,
            )),
        },
    ];
    let mut actual = vec![];
    let updater = |r| actual.push(r);

    batch_tendril_action(
        updater,
        ActionMode::Out,
        &given_td_repo,
        given,
        dry_run,
        force,
    );

    assert_eq!(actual, expected);

    if dry_run {
        assert!(!remote_app1_file.exists());
        assert!(!remote_app2_file.exists());
        assert!(!remote_app3_fileb_pa.exists());
        assert!(!remote_app3_fileb_pb.exists());
        assert!(!remote_app1_nested_file.exists());
    }
    else {
        let remote_app1_file_contents =
            read_to_string(&remote_app1_file).unwrap();
        let remote_app2_file_contents =
            read_to_string(&remote_app2_file).unwrap();
        let remote_app3_fileb_pa_contents =
            read_to_string(&remote_app3_fileb_pa).unwrap();
        let remote_app3_fileb_pb_contents =
            read_to_string(&remote_app3_fileb_pb).unwrap();
        let remote_app1_nested_file_contents =
            read_to_string(&remote_app1_nested_file).unwrap();

        assert_eq!(remote_app1_file_contents, "Local app 1 file contents");
        assert_eq!(remote_app2_file_contents, "Local app 2 file contents");
        assert_eq!(
            remote_app3_fileb_pa_contents,
            "Local app 3 file b contents"
        );
        assert_eq!(
            remote_app3_fileb_pb_contents,
            "Local app 3 file b contents"
        );
        assert_eq!(
            remote_app1_nested_file_contents,
            "Local app 1 nested file contents"
        );
        assert!(remote_app1_file.is_symlink());
        assert!(!remote_app2_file.is_symlink());
        assert!(!remote_app3_fileb_pa.is_symlink());
        assert!(!remote_app3_fileb_pb.is_symlink());
        assert!(remote_app1_dir.is_symlink());
    }
}

#[rstest]
#[serial("mut-env-var-testing")]
fn parent_path_vars_are_resolved(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let setup = Setup::new();
    setup.make_td_repo_dir();
    let mut tendril = setup.file_tendril_bundle();
    tendril.link = mode == ActionMode::Link;
    set_parents(&mut tendril, &[PathBuf::from("~/I_do_not_exist/<var>/")]);
    let tendrils = vec![tendril.clone()];
    std::env::set_var("HOME", "My/Home");
    std::env::set_var("var", "value");

    use std::path::MAIN_SEPARATOR;
    let expected_resolved_path = format!(
        "My{MAIN_SEPARATOR}Home{MAIN_SEPARATOR}I_do_not_exist{MAIN_SEPARATOR}value{MAIN_SEPARATOR}misc.txt"
    );
    let expected_loc = match mode {
        ActionMode::Pull => Location::Source,
        _ => Location::Dest,
    };
    let expected = vec![TendrilReport {
        orig_tendril: Rc::new(tendril),
        name: "misc.txt".to_string(),
        log: Ok(ActionLog::new(
            None,
            None,
            PathBuf::from(expected_resolved_path.clone()),
            Err(TendrilActionError::IoError {
                kind: std::io::ErrorKind::NotFound,
                loc: expected_loc,
            }),
        )),
    }];

    let mut actual = vec![];
    let updater = |r| actual.push(r);

    batch_tendril_action(
        updater,
        mode,
        &setup.td_repo,
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
