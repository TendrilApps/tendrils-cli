use crate::test_utils::{get_disposable_dir, is_empty, set_parents, Setup};
use crate::{
    tendril_action,
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
    let given_td_dir = temp_parent_dir.path().join("TendrilsDir");

    let actual = tendril_action(mode, &given_td_dir, &[], dry_run, force);

    assert!(actual.is_empty());
    assert!(is_empty(&given_td_dir))
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
    let local_app1_nested_file = local_app1_dir.join("nested1.txt");
    let local_app2_file = given_td_dir.join("App2").join("misc2.txt");
    let local_app3_file_b = given_td_dir.join("App3").join("misc3.txt");
    create_dir_all(&given_td_dir).unwrap();
    create_dir_all(&remote_app1_dir).unwrap();
    write(&remote_app1_file, "Remote app 1 file contents").unwrap();
    write(&remote_app2_file, "Remote app 2 file contents").unwrap();
    write(&remote_app3_fileb_pa, "Remote app 3 file b parent a contents")
        .unwrap();
    write(&remote_app1_nested_file, "Remote app 1 nested file contents")
        .unwrap();

    let mut given = [
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
            orig_tendril: &given[0],
            name: &given[0].names[0],
            log: Ok(ActionLog::new(
                None,
                Some(FsoType::File),
                remote_app1_file,
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[1],
            name: &given[1].names[0],
            log: Ok(ActionLog::new(
                None,
                Some(FsoType::File),
                remote_app2_file,
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[2],
            name: &given[2].names[0],
            log: Ok(ActionLog::new(
                None,
                Some(FsoType::Dir),
                remote_app1_dir,
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[3],
            name: &given[3].names[0],
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
            orig_tendril: &given[3],
            name: &given[3].names[1],
            log: Ok(ActionLog::new(
                None,
                Some(FsoType::File),
                remote_app3_fileb_pa,
                expected_success,
            )),
        },
        // The second path should not be considered since this is a pull action
    ];

    let actual =
        tendril_action(ActionMode::Pull, &given_td_dir, &given, dry_run, force);

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
    let local_nested_app1_file = local_app1_dir.join("nested1.txt");
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
            orig_tendril: &given[0],
            name: &given[0].names[0],
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app1_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[1],
            name: &given[1].names[0],
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app2_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[2],
            name: &given[2].names[0],
            log: Ok(ActionLog::new(
                Some(FsoType::Dir),
                None,
                remote_app1_dir.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[3],
            name: &given[3].names[0],
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
            orig_tendril: &given[3],
            name: &given[3].names[0],
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
            orig_tendril: &given[3],
            name: &given[3].names[1],
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app3_fileb_pa.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[3],
            name: &given[3].names[1],
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app3_fileb_pb.clone(),
                expected_success,
            )),
        },
    ];

    let actual =
        tendril_action(ActionMode::Push, &given_td_dir, &given, dry_run, force);

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
    let local_nested_app1_file = local_app1_dir.join("nested1.txt");
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
            orig_tendril: &given[0],
            name: &given[0].names[0],
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app1_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[1],
            name: &given[1].names[0],
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app2_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[2],
            name: &given[2].names[0],
            log: Ok(ActionLog::new(
                Some(FsoType::Dir),
                None,
                remote_app1_dir.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[3],
            name: &given[3].names[0],
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
            orig_tendril: &given[3],
            name: &given[3].names[0],
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
            orig_tendril: &given[3],
            name: &given[3].names[1],
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app3_fileb_pa.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[3],
            name: &given[3].names[1],
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app3_fileb_pb.clone(),
                expected_success,
            )),
        },
    ];

    let actual =
        tendril_action(ActionMode::Link, &given_td_dir, &given, dry_run, force);

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
    let local_nested_app1_file = local_app1_dir.join("nested1.txt");
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
            orig_tendril: &given[0],
            name: &given[0].names[0],
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app1_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[1],
            name: &given[1].names[0],
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app2_file.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[2],
            name: &given[2].names[0],
            log: Ok(ActionLog::new(
                Some(FsoType::Dir),
                None,
                remote_app1_dir.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[3],
            name: &given[3].names[0],
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
            orig_tendril: &given[3],
            name: &given[3].names[0],
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
            orig_tendril: &given[3],
            name: &given[3].names[1],
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app3_fileb_pa.clone(),
                expected_success.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: &given[3],
            name: &given[3].names[1],
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                remote_app3_fileb_pb.clone(),
                expected_success,
            )),
        },
    ];

    let actual =
        tendril_action(ActionMode::Out, &given_td_dir, &given, dry_run, force);

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
    setup.make_td_dir();
    let mut tendril = setup.file_tendril_bundle();
    tendril.link = mode == ActionMode::Link;
    set_parents(&mut tendril, &[PathBuf::from("~/I_do_not_exist/<var>/")]);
    let tendrils = [tendril.clone()];
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
        orig_tendril: &tendril,
        name: "misc.txt",
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

    let actual = tendril_action(mode, &setup.td_dir, &tendrils, dry_run, force);

    let actual_result_path = &actual[0].log.as_ref().unwrap().resolved_path();

    let actual_resolved_path_str = actual_result_path.to_string_lossy();
    assert_eq!(actual_resolved_path_str.into_owned(), expected_resolved_path);
    assert_eq!(actual, expected);
}

// TODO: Test when the second tendril is a parent/child to the first tendril
