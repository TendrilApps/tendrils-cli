use crate::{tendril_action, TendrilActionError, TendrilActionSuccess};
use crate::enums::ActionMode;
use crate::tendril_bundle::TendrilBundle;
use crate::tendril_action_report::TendrilActionReport;
use crate::test_utils::{
    get_disposable_dir,
    is_empty,
    set_parents,
    Setup
};
use fs_extra::file::read_to_string;
use rstest::rstest;
use serial_test::serial;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;
use tempdir::TempDir;

#[rstest]
fn given_empty_list_returns_empty(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let given_td_dir = temp_parent_dir.path().join("TendrilsDir");

    let actual = tendril_action(
        mode,
        &given_td_dir,
        &[],
        dry_run,
        force,
    );

    assert!(actual.is_empty());
    assert!(is_empty(&given_td_dir))
}

#[rstest]
#[case(true)]
#[case(false)]
fn pull_returns_tendril_and_result_for_each_given(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_grandparent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let given_td_dir = temp_grandparent_dir.path().join("TendrilsDir");
    let given_parent_dir_a = temp_grandparent_dir.path().join("ParentA");
    let given_parent_dir_b = temp_grandparent_dir.path().join("ParentB");
    let remote_app1_file = given_parent_dir_a.join("misc1.txt");
    let remote_app1_dir = given_parent_dir_a.join("App1 Dir");
    let remote_app1_nested_file = remote_app1_dir.join("nested1.txt");
    let remote_app2_file = given_parent_dir_a.join("misc2.txt");
    let remote_app3_fileb_pa = given_parent_dir_a.join("misc3.txt");
    let local_app1_file = given_td_dir.join("App1").join("misc1.txt");
    let local_app1_dir = given_td_dir.join("App1").join("App1 Dir");
    let local_app1_nested_file= local_app1_dir.join("nested1.txt");
    let local_app2_file = given_td_dir.join("App2").join("misc2.txt");
    let local_app3_file_b = given_td_dir.join("App3").join("misc3.txt");
    create_dir_all(&given_td_dir).unwrap();
    create_dir_all(&remote_app1_dir).unwrap();
    write(&remote_app1_file, "Remote app 1 file contents").unwrap();
    write(&remote_app2_file, "Remote app 2 file contents").unwrap();
    write(&remote_app3_fileb_pa, "Remote app 3 file b parent a contents").unwrap();
    write(&remote_app1_nested_file, "Remote app 1 nested file contents").unwrap();

    let mut given = [
        TendrilBundle::new("App1", vec!["misc1.txt"]),
        TendrilBundle::new("App2", vec!["misc2.txt"]),
        TendrilBundle::new("App1", vec!["App1 Dir"]),
        TendrilBundle::new("App3", vec!["I don't exist", "misc3.txt"]),
    ];

    set_parents(&mut given[0], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[1], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[2], &[given_parent_dir_a.clone()]);
    set_parents(
        &mut given[3],
        &[given_parent_dir_a.clone(), given_parent_dir_b.clone()]
    );

    let io_not_found_err = std::io::Error::from(std::io::ErrorKind::NotFound);
    let expected: Vec<TendrilActionReport> = match dry_run {
        true => {
            vec![
                TendrilActionReport {
                    orig_tendril: &given[0],
                    name: &given[0].names[0],
                    resolved_path: Ok(remote_app1_file),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
                TendrilActionReport {
                    orig_tendril: &given[1],
                    name: &given[1].names[0],
                    resolved_path: Ok(remote_app2_file),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
                TendrilActionReport {
                    orig_tendril: &given[2],
                    name: &given[2].names[0],
                    resolved_path: Ok(remote_app1_dir),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[0],
                    resolved_path: Ok(given_parent_dir_a.join("I don't exist")),
                    action_result: Some(Err(TendrilActionError::IoError(io_not_found_err))),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[1],
                    resolved_path: Ok(remote_app3_fileb_pa),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
                // The second path should not be considered since this is a pull action
            ]
        },
        false => {
            vec![
                TendrilActionReport {
                    orig_tendril: &given[0],
                    name: &given[0].names[0],
                    resolved_path: Ok(remote_app1_file),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
                TendrilActionReport {
                    orig_tendril: &given[1],
                    name: &given[1].names[0],
                    resolved_path: Ok(remote_app2_file),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
                TendrilActionReport {
                    orig_tendril: &given[2],
                    name: &given[2].names[0],
                    resolved_path: Ok(remote_app1_dir),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[0],
                    resolved_path: Ok(given_parent_dir_a.join("I don't exist")),
                    action_result: Some(Err(TendrilActionError::IoError(io_not_found_err))),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[1],
                    resolved_path: Ok(remote_app3_fileb_pa),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
            ]
        }
    };

    let actual = tendril_action(
        ActionMode::Pull,
        &given_td_dir,
        &given,
        dry_run,
        force,
    );

    for (i, actual_report) in actual.iter().enumerate() {
        assert_eq!(actual_report.orig_tendril, expected[i].orig_tendril);
        assert_eq!(actual_report.name, expected[i].name);
        assert_eq!(actual_report.resolved_path, expected[i].resolved_path);
    }

    // TendrilActionError is not equatable so must match manually
    if dry_run {
        assert!(matches!(actual[0].action_result, Some(Ok(TendrilActionSuccess::Skipped))));
        assert!(matches!(actual[1].action_result, Some(Ok(TendrilActionSuccess::Skipped))));
        assert!(matches!(actual[2].action_result, Some(Ok(TendrilActionSuccess::Skipped))));
        assert!(matches!(actual[4].action_result, Some(Ok(TendrilActionSuccess::Skipped))));

        assert!(!local_app1_file.exists());
        assert!(!local_app1_dir.exists());
        assert!(!local_app2_file.exists());
        assert!(!local_app3_file_b.exists());
        assert!(!local_app1_nested_file.exists());
    }
    else {
        assert!(matches!(actual[0].action_result, Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[1].action_result, Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[2].action_result, Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[4].action_result, Some(Ok(TendrilActionSuccess::Ok))));

        let local_app1_file_contents = read_to_string(local_app1_file).unwrap();
        let local_app2_file_contents = read_to_string(local_app2_file).unwrap();
        let local_app3_fileb_contents = read_to_string(local_app3_file_b).unwrap();
        let local_app1_nested_file_contents = read_to_string(
            local_app1_nested_file
        ).unwrap();

        assert_eq!(local_app1_file_contents, "Remote app 1 file contents");
        assert!(local_app1_dir.exists());
        assert_eq!(local_app2_file_contents, "Remote app 2 file contents");
        assert_eq!(local_app3_fileb_contents, "Remote app 3 file b parent a contents");
        assert_eq!(local_app1_nested_file_contents, "Remote app 1 nested file contents");
    }

    match &actual[3].action_result {
        Some(Err(TendrilActionError::IoError(e))) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound)
        },
        _ => panic!()
    }
    assert_eq!(actual.len(), expected.len());
}

#[rstest]
#[case(true)]
#[case(false)]
fn push_returns_tendril_and_result_for_each_given(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_grandparent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let given_td_dir = temp_grandparent_dir.path().join("TendrilsDir");
    let given_parent_dir_a = temp_grandparent_dir.path().join("ParentA");
    let given_parent_dir_b = temp_grandparent_dir.path().join("ParentB");
    let remote_app1_file = given_parent_dir_a.join("misc1.txt");
    let remote_app1_dir = given_parent_dir_a.join("App1 Dir");
    let remote_app1_nested_file = remote_app1_dir.join("nested1.txt");
    let remote_app2_file = given_parent_dir_a.join("misc2.txt");
    let remote_app3_fileb_pa = given_parent_dir_a.join("misc3.txt");
    let remote_app3_fileb_pb = given_parent_dir_b.join("misc3.txt");
    let local_app1_file = given_td_dir.join("App1").join("misc1.txt");
    let local_app1_dir = given_td_dir.join("App1").join("App1 Dir");
    let local_nested_app1_file= local_app1_dir.join("nested1.txt");
    let local_app2_file = given_td_dir.join("App2").join("misc2.txt");
    let local_app3_file_b = given_td_dir.join("App3").join("misc3.txt");
    create_dir_all(given_parent_dir_a.clone()).unwrap();
    create_dir_all(given_parent_dir_b.clone()).unwrap();
    create_dir_all(&local_app1_dir).unwrap();
    create_dir_all(&given_td_dir.join("App2")).unwrap();
    create_dir_all(&given_td_dir.join("App3")).unwrap();
    write(&local_app1_file, "Local app 1 file contents").unwrap();
    write(&local_app2_file, "Local app 2 file contents").unwrap();
    write(&local_app3_file_b, "Local app 3 file b contents").unwrap();
    write(&local_nested_app1_file, "Local app 1 nested file contents").unwrap();

    let mut given = [
        TendrilBundle::new("App1", vec!["misc1.txt"]),
        TendrilBundle::new("App2", vec!["misc2.txt"]),
        TendrilBundle::new("App1", vec!["App1 Dir"]),
        TendrilBundle::new("App3", vec!["I don't exist", "misc3.txt"]),
    ];

    set_parents(&mut given[0], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[1], &[given_parent_dir_a.clone()]);
    set_parents(&mut given[2], &[given_parent_dir_a.clone()]);
    set_parents(
        &mut given[3],
        &[given_parent_dir_a.clone(), given_parent_dir_b.clone()]
    );

    let io_not_found_err_1 = std::io::Error::from(std::io::ErrorKind::NotFound);
    let io_not_found_err_2 = std::io::Error::from(std::io::ErrorKind::NotFound);
    let expected: Vec<TendrilActionReport> = match dry_run {
        true => {
            vec![
                TendrilActionReport {
                    orig_tendril: &given[0],
                    name: &given[0].names[0],
                    resolved_path: Ok(remote_app1_file.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
                TendrilActionReport {
                    orig_tendril: &given[1],
                    name: &given[1].names[0],
                    resolved_path: Ok(remote_app2_file.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
                TendrilActionReport {
                    orig_tendril: &given[2],
                    name: &given[2].names[0],
                    resolved_path: Ok(remote_app1_dir.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[0],
                    resolved_path: Ok(given_parent_dir_a.join("I don't exist")),
                    action_result: Some(Err(TendrilActionError::IoError(io_not_found_err_1))),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[0],
                    resolved_path: Ok(given_parent_dir_b.join("I don't exist")),
                    action_result: Some(Err(TendrilActionError::IoError(io_not_found_err_2))),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[1],
                    resolved_path: Ok(remote_app3_fileb_pa.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[1],
                    resolved_path: Ok(remote_app3_fileb_pb.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
            ]
        },
        false => {
            vec![
                TendrilActionReport {
                    orig_tendril: &given[0],
                    name: &given[0].names[0],
                    resolved_path: Ok(remote_app1_file.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
                TendrilActionReport {
                    orig_tendril: &given[1],
                    name: &given[1].names[0],
                    resolved_path: Ok(remote_app2_file.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
                TendrilActionReport {
                    orig_tendril: &given[2],
                    name: &given[2].names[0],
                    resolved_path: Ok(remote_app1_dir.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[0],
                    resolved_path: Ok(given_parent_dir_a.join("I don't exist")),
                    action_result: Some(Err(TendrilActionError::IoError(io_not_found_err_1))),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[0],
                    resolved_path: Ok(given_parent_dir_b.join("I don't exist")),
                    action_result: Some(Err(TendrilActionError::IoError(io_not_found_err_2))),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[1],
                    resolved_path: Ok(remote_app3_fileb_pa.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[1],
                    resolved_path: Ok(remote_app3_fileb_pb.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
            ]
        }
    };

    let actual = tendril_action(
        ActionMode::Push,
        &given_td_dir,
        &given,
        dry_run,
        force,
    );

    for (i, actual_report) in actual.iter().enumerate() {
        assert_eq!(actual_report.orig_tendril, expected[i].orig_tendril);
        assert_eq!(actual_report.name, expected[i].name);
        assert_eq!(actual_report.resolved_path, expected[i].resolved_path);
    }

    // TendrilActionError is not equatable so must match manually
    if dry_run {
        assert!(matches!(actual[0].action_result, Some(Ok(TendrilActionSuccess::Skipped))));
        assert!(matches!(actual[1].action_result, Some(Ok(TendrilActionSuccess::Skipped))));
        assert!(matches!(actual[2].action_result, Some(Ok(TendrilActionSuccess::Skipped))));
        assert!(matches!(actual[5].action_result, Some(Ok(TendrilActionSuccess::Skipped))));
        assert!(matches!(actual[6].action_result, Some(Ok(TendrilActionSuccess::Skipped))));

        assert!(!remote_app1_file.exists());
        assert!(!remote_app2_file.exists());
        assert!(!remote_app3_fileb_pa.exists());
        assert!(!remote_app3_fileb_pb.exists());
        assert!(!remote_app1_nested_file.exists());
    }
    else {
        assert!(matches!(actual[0].action_result, Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[1].action_result, Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[2].action_result, Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[5].action_result, Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[6].action_result, Some(Ok(TendrilActionSuccess::Ok))));

        let remote_app1_file_contents = read_to_string(&remote_app1_file).unwrap();
        let remote_app2_file_contents = read_to_string(&remote_app2_file).unwrap();
        let remote_app3_fileb_pa_contents = read_to_string(&remote_app3_fileb_pa).unwrap();
        let remote_app3_fileb_pb_contents = read_to_string(&remote_app3_fileb_pb).unwrap();
        let remote_app1_nested_file_contents = read_to_string(
            &remote_app1_nested_file
        ).unwrap();

        assert_eq!(remote_app1_file_contents, "Local app 1 file contents");
        assert_eq!(remote_app2_file_contents, "Local app 2 file contents");
        assert_eq!(remote_app3_fileb_pa_contents, "Local app 3 file b contents");
        assert_eq!(remote_app3_fileb_pb_contents, "Local app 3 file b contents");
        assert_eq!(remote_app1_nested_file_contents, "Local app 1 nested file contents");
        assert!(!remote_app1_file.is_symlink());
        assert!(!remote_app2_file.is_symlink());
        assert!(!remote_app3_fileb_pa.is_symlink());
        assert!(!remote_app3_fileb_pb.is_symlink());
        assert!(!remote_app1_dir.is_symlink());
    }

    match &actual[3].action_result {
        Some(Err(TendrilActionError::IoError(e))) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound)
        },
        _ => panic!()
    }
    match &actual[4].action_result {
        Some(Err(TendrilActionError::IoError(e))) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound)
        },
        _ => panic!()
    }
    assert_eq!(actual.len(), expected.len());
}

#[rstest]
#[case(true)]
#[case(false)]
fn link_returns_tendril_and_result_for_each_given(
    #[case] dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let temp_grandparent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let given_td_dir = temp_grandparent_dir.path().join("TendrilsDir");
    let given_parent_dir_a = temp_grandparent_dir.path().join("ParentA");
    let given_parent_dir_b = temp_grandparent_dir.path().join("ParentB");
    let remote_app1_file = given_parent_dir_a.join("misc1.txt");
    let remote_app1_dir = given_parent_dir_a.join("App1 Dir");
    let remote_app1_nested_file = remote_app1_dir.join("nested1.txt");
    let remote_app2_file = given_parent_dir_a.join("misc2.txt");
    let remote_app3_fileb_pa = given_parent_dir_a.join("misc3.txt");
    let remote_app3_fileb_pb = given_parent_dir_b.join("misc3.txt");
    let local_app1_file = given_td_dir.join("App1").join("misc1.txt");
    let local_app1_dir = given_td_dir.join("App1").join("App1 Dir");
    let local_nested_app1_file= local_app1_dir.join("nested1.txt");
    let local_app2_file = given_td_dir.join("App2").join("misc2.txt");
    let local_app3_file_b = given_td_dir.join("App3").join("misc3.txt");
    create_dir_all(given_parent_dir_a.clone()).unwrap();
    create_dir_all(given_parent_dir_b.clone()).unwrap();
    create_dir_all(&local_app1_dir).unwrap();
    create_dir_all(&given_td_dir.join("App2")).unwrap();
    create_dir_all(&given_td_dir.join("App3")).unwrap();
    write(&local_app1_file, "Local app 1 file contents").unwrap();
    write(&local_app2_file, "Local app 2 file contents").unwrap();
    write(&local_app3_file_b, "Local app 3 file b contents").unwrap();
    write(&local_nested_app1_file, "Local app 1 nested file contents").unwrap();

    let mut given = [
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
    set_parents(
        &mut given[3],
        &[given_parent_dir_a.clone(), given_parent_dir_b.clone()]
    );

    let io_not_found_err_1 = std::io::Error::from(std::io::ErrorKind::NotFound);
    let io_not_found_err_2 = std::io::Error::from(std::io::ErrorKind::NotFound);
    let expected: Vec<TendrilActionReport> = match dry_run {
        true => {
            vec![
                TendrilActionReport {
                    orig_tendril: &given[0],
                    name: &given[0].names[0],
                    resolved_path: Ok(remote_app1_file.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
                TendrilActionReport {
                    orig_tendril: &given[1],
                    name: &given[1].names[0],
                    resolved_path: Ok(remote_app2_file.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
                TendrilActionReport {
                    orig_tendril: &given[2],
                    name: &given[2].names[0],
                    resolved_path: Ok(remote_app1_dir.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[0],
                    resolved_path: Ok(given_parent_dir_a.join("I don't exist")),
                    action_result: Some(Err(TendrilActionError::IoError(io_not_found_err_1))),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[0],
                    resolved_path: Ok(given_parent_dir_b.join("I don't exist")),
                    action_result: Some(Err(TendrilActionError::IoError(io_not_found_err_2))),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[1],
                    resolved_path: Ok(remote_app3_fileb_pa.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[1],
                    resolved_path: Ok(remote_app3_fileb_pb.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
            ]
        },
        false => {
            vec![
                TendrilActionReport {
                    orig_tendril: &given[0],
                    name: &given[0].names[0],
                    resolved_path: Ok(remote_app1_file.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
                TendrilActionReport {
                    orig_tendril: &given[1],
                    name: &given[1].names[0],
                    resolved_path: Ok(remote_app2_file.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
                TendrilActionReport {
                    orig_tendril: &given[2],
                    name: &given[2].names[0],
                    resolved_path: Ok(remote_app1_dir.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[0],
                    resolved_path: Ok(given_parent_dir_a.join("I don't exist")),
                    action_result: Some(Err(TendrilActionError::IoError(io_not_found_err_1))),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[0],
                    resolved_path: Ok(given_parent_dir_b.join("I don't exist")),
                    action_result: Some(Err(TendrilActionError::IoError(io_not_found_err_2))),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[1],
                    resolved_path: Ok(remote_app3_fileb_pa.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    name: &given[3].names[1],
                    resolved_path: Ok(remote_app3_fileb_pb.clone()),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
            ]
        }
    };

    let actual = tendril_action(
        ActionMode::Link,
        &given_td_dir,
        &given,
        dry_run,
        force,
    );

    for (i, actual_report) in actual.iter().enumerate() {
        assert_eq!(actual_report.orig_tendril, expected[i].orig_tendril);
        assert_eq!(actual_report.name, expected[i].name);
        assert_eq!(actual_report.resolved_path, expected[i].resolved_path);
    }

    // TendrilActionError is not equatable so must match manually
    if dry_run {
        assert!(matches!(actual[0].action_result, Some(Ok(TendrilActionSuccess::Skipped))));
        assert!(matches!(actual[1].action_result, Some(Ok(TendrilActionSuccess::Skipped))));
        assert!(matches!(actual[2].action_result, Some(Ok(TendrilActionSuccess::Skipped))));
        assert!(matches!(actual[5].action_result, Some(Ok(TendrilActionSuccess::Skipped))));
        assert!(matches!(actual[6].action_result, Some(Ok(TendrilActionSuccess::Skipped))));

        assert!(!remote_app1_file.exists());
        assert!(!remote_app2_file.exists());
        assert!(!remote_app3_fileb_pa.exists());
        assert!(!remote_app3_fileb_pb.exists());
        assert!(!remote_app1_nested_file.exists());
    }
    else {
        assert!(matches!(actual[0].action_result, Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[1].action_result, Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[2].action_result, Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[5].action_result, Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[6].action_result, Some(Ok(TendrilActionSuccess::Ok))));

        let remote_app1_file_contents = read_to_string(&remote_app1_file).unwrap();
        let remote_app2_file_contents = read_to_string(&remote_app2_file).unwrap();
        let remote_app3_fileb_pa_contents = read_to_string(&remote_app3_fileb_pa).unwrap();
        let remote_app3_fileb_pb_contents = read_to_string(&remote_app3_fileb_pb).unwrap();
        let remote_app1_nested_file_contents = read_to_string(
            &remote_app1_nested_file
        ).unwrap();

        assert_eq!(remote_app1_file_contents, "Local app 1 file contents");
        assert_eq!(remote_app2_file_contents, "Local app 2 file contents");
        assert_eq!(remote_app3_fileb_pa_contents, "Local app 3 file b contents");
        assert_eq!(remote_app3_fileb_pb_contents, "Local app 3 file b contents");
        assert_eq!(remote_app1_nested_file_contents, "Local app 1 nested file contents");
        assert!(remote_app1_file.is_symlink());
        assert!(remote_app2_file.is_symlink());
        assert!(remote_app3_fileb_pa.is_symlink());
        assert!(remote_app3_fileb_pb.is_symlink());
        assert!(remote_app1_dir.is_symlink());
    }

    match &actual[3].action_result {
        Some(Err(TendrilActionError::IoError(e))) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound)
        },
        _ => panic!()
    }
    match &actual[4].action_result {
        Some(Err(TendrilActionError::IoError(e))) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound)
        },
        _ => panic!()
    }
    assert_eq!(actual.len(), expected.len());
}


#[rstest]
#[serial("mut-env-var-testing")]
fn parent_path_vars_are_resolved(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    let mut tendril = setup.file_tendril_bundle();
    tendril.link = mode == ActionMode::Link;
    set_parents(
        &mut tendril, &[PathBuf::from("~/I_do_not_exist/<var>/")]
    );
    let tendrils = [tendril];
    std::env::set_var("HOME", "My/Home");
    std::env::set_var("var", "value");

    use std::path::MAIN_SEPARATOR;
    let expected_resolved_path = format!(
        "My{MAIN_SEPARATOR}Home{MAIN_SEPARATOR}I_do_not_exist{MAIN_SEPARATOR}value{MAIN_SEPARATOR}misc.txt"
    );

    let actual = tendril_action(
        mode,
        &setup.td_dir,
        &tendrils,
        dry_run,
        force,
    );

    let actual_resolved_path = actual[0].resolved_path
        .as_ref()
        .unwrap()
        .to_string_lossy();
    assert_eq!(actual_resolved_path.into_owned(), expected_resolved_path);
    match actual[0].action_result.as_ref() {
        Some(Err(TendrilActionError::IoError(e))) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        },
        Some(e) => assert_eq!(format!("{:?}", e), "dsfsd"),
        _ => panic!()
    }
}

// TODO: Test when the second tendril is a parent/child to the first tendril
