use crate::{tendril_action, TendrilActionError, TendrilActionSuccess};
use crate::action_mode::ActionMode;
use crate::tendril::Tendril;
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
    let source_app1_file = given_parent_dir_a.join("misc1.txt");
    let source_app2_file = given_parent_dir_a.join("misc2.txt");
    let source_app3_file_a = given_parent_dir_a.join("misc3.txt");
    let source_app1_dir = given_parent_dir_a.join("App1 Dir");
    let nested_app1_file = source_app1_dir.join("nested1.txt");
    let dest_app1_file = given_td_dir.join("App1").join("misc1.txt");
    let dest_app2_file = given_td_dir.join("App2").join("misc2.txt");
    let dest_app3_file_b = given_td_dir.join("App2").join("misc3.txt");
    let dest_app1_nested= given_td_dir.join("App1").join("App1 Dir").join("nested1.txt");
    create_dir_all(&source_app1_dir).unwrap();
    write(&source_app1_file, "App 1 file contents").unwrap();
    write(&source_app2_file, "App 2 file contents").unwrap();
    write(&source_app3_file_a, "App 3 file a contents").unwrap();
    write(&nested_app1_file, "Nested 1 file contents").unwrap();

    let mut given = [
        Tendril::new("App1", vec!["misc1.txt"]),
        Tendril::new("App2", vec!["misc2.txt"]),
        Tendril::new("App1", vec!["App1 Dir"]),
        Tendril::new("App3", vec!["I don't exist", "misc3.txt"]),
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
                    resolved_path: Ok(source_app1_file),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
                TendrilActionReport {
                    orig_tendril: &given[1],
                    name: &given[1].names[0],
                    resolved_path: Ok(source_app2_file),
                    action_result: Some(Ok(TendrilActionSuccess::Skipped)),
                },
                TendrilActionReport {
                    orig_tendril: &given[2],
                    name: &given[2].names[0],
                    resolved_path: Ok(source_app1_dir),
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
                    resolved_path: Ok(source_app3_file_a),
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
                    resolved_path: Ok(source_app1_file),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
                TendrilActionReport {
                    orig_tendril: &given[1],
                    name: &given[1].names[0],
                    resolved_path: Ok(source_app2_file),
                    action_result: Some(Ok(TendrilActionSuccess::Ok)),
                },
                TendrilActionReport {
                    orig_tendril: &given[2],
                    name: &given[2].names[0],
                    resolved_path: Ok(source_app1_dir),
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
                    resolved_path: Ok(source_app3_file_a),
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

        assert!(!dest_app1_file.exists());
        assert!(!dest_app2_file.exists());
        assert!(!dest_app3_file_b.exists());
        assert!(!dest_app1_nested.exists());
    }
    else {
        assert!(matches!(actual[0].action_result, Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[1].action_result, Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[2].action_result, Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[4].action_result, Some(Ok(TendrilActionSuccess::Ok))));

        let dest_app1_file_contents = read_to_string(dest_app1_file).unwrap();
        let dest_app2_file_contents = read_to_string(dest_app2_file).unwrap();
        let dest_app1_nested_file_contents = read_to_string(
            dest_app1_nested
        ).unwrap();

        assert_eq!(dest_app1_file_contents, "App 1 file contents");
        assert_eq!(dest_app2_file_contents, "App 2 file contents");
        assert_eq!(dest_app1_nested_file_contents, "Nested 1 file contents");
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
    let mut tendril = setup.file_tendril();
    tendril.link = mode == ActionMode::Link;
    set_parents(
        &mut tendril, &[PathBuf::from("~/I_do_not_exist/<var>/")]
    );
    let tendrils = [tendril];
    std::env::set_var("HOME", "My/Home");
    std::env::set_var("var", "value");
    
    let expected_resolved_path = "My/Home/I_do_not_exist/value/misc.txt";

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
