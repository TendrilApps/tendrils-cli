use crate::{tendril_action, TendrilActionError, TendrilActionSuccess};
use crate::action_mode::ActionMode;
use crate::tendril::Tendril;
use crate::tendril_action_report::TendrilActionReport;
use crate::test_utils::{
    get_disposable_dir,
    is_empty,
    set_all_platform_paths,
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
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();
    let given_td_dir = temp_parent_dir.path().join("TendrilsDir");
    let given_parent_dir = temp_parent_dir.path().to_path_buf();
    let source_app1_file = given_parent_dir.join("misc1.txt");
    let source_app2_file = given_parent_dir.join("misc2.txt");
    let source_app1_dir = given_parent_dir.join("App1 Dir");
    let nested_app1_file = source_app1_dir.join("nested1.txt");
    let dest_app1_file = given_td_dir.join("App1").join("misc1.txt");
    let dest_app2_file = given_td_dir.join("App2").join("misc2.txt");
    let dest_app1_nested= given_td_dir.join("App1").join("App1 Dir").join("nested1.txt");
    create_dir_all(&source_app1_dir).unwrap();
    write(&source_app1_file, "App 1 file contents").unwrap();
    write(&source_app2_file, "App 2 file contents").unwrap();
    write(&nested_app1_file, "Nested 1 file contents").unwrap();

    let mut given = [
        Tendril::new("App1", "misc1.txt"),
        Tendril::new("App2", "misc2.txt"),
        Tendril::new("App1", "App1 Dir"),
        Tendril::new("App2", "I don't exist"),
    ];

    set_all_platform_paths(&mut given[0], &[given_parent_dir.clone()]);
    set_all_platform_paths(&mut given[1], &[given_parent_dir.clone()]);
    set_all_platform_paths(&mut given[2], &[given_parent_dir.clone()]);
    set_all_platform_paths(&mut given[3], &[given_parent_dir.clone()]);

    let io_not_found_err = std::io::Error::from(std::io::ErrorKind::NotFound);
    let expected: Vec<TendrilActionReport> = match dry_run {
        true => {
            vec![
                TendrilActionReport {
                    orig_tendril: &given[0],
                    resolved_paths: vec![Ok(source_app1_file)],
                    action_results: vec![Some(Ok(TendrilActionSuccess::Skipped))],
                },
                TendrilActionReport {
                    orig_tendril: &given[1],
                    resolved_paths: vec![Ok(source_app2_file)],
                    action_results: vec![Some(Ok(TendrilActionSuccess::Skipped))],
                },
                TendrilActionReport {
                    orig_tendril: &given[2],
                    resolved_paths: vec![Ok(source_app1_dir)],
                    action_results: vec![Some(Ok(TendrilActionSuccess::Skipped))],
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    resolved_paths: vec![Ok(given_parent_dir.join("I don't exist"))],
                    action_results: vec![Some(Err(TendrilActionError::IoError(io_not_found_err)))],
                },
            ]
        },
        false => {
            vec![
                TendrilActionReport {
                    orig_tendril: &given[0],
                    resolved_paths: vec![Ok(source_app1_file)],
                    action_results: vec![Some(Ok(TendrilActionSuccess::Ok))],
                },
                TendrilActionReport {
                    orig_tendril: &given[1],
                    resolved_paths: vec![Ok(source_app2_file)],
                    action_results: vec![Some(Ok(TendrilActionSuccess::Ok))],
                },
                TendrilActionReport {
                    orig_tendril: &given[2],
                    resolved_paths: vec![Ok(source_app1_dir)],
                    action_results: vec![Some(Ok(TendrilActionSuccess::Ok))],
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    resolved_paths: vec![Ok(given_parent_dir.join("I don't exist"))],
                    action_results: vec![Some(Err(TendrilActionError::IoError(io_not_found_err)))],
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
        assert_eq!(actual_report.resolved_paths, expected[i].resolved_paths);
    }

    // TendrilActionError is not equatable so must match manually
    if dry_run {
        assert!(matches!(actual[0].action_results[0], Some(Ok(TendrilActionSuccess::Skipped))));
        assert!(matches!(actual[1].action_results[0], Some(Ok(TendrilActionSuccess::Skipped))));
        assert!(matches!(actual[2].action_results[0], Some(Ok(TendrilActionSuccess::Skipped))));

        assert!(!dest_app1_file.exists());
        assert!(!dest_app2_file.exists());
        assert!(!dest_app1_nested.exists());
    }
    else {
        assert!(matches!(actual[0].action_results[0], Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[1].action_results[0], Some(Ok(TendrilActionSuccess::Ok))));
        assert!(matches!(actual[2].action_results[0], Some(Ok(TendrilActionSuccess::Ok))));

        let dest_app1_file_contents = read_to_string(dest_app1_file).unwrap();
        let dest_app2_file_contents = read_to_string(dest_app2_file).unwrap();
        let dest_app1_nested_file_contents = read_to_string(
            dest_app1_nested
        ).unwrap();

        assert_eq!(dest_app1_file_contents, "App 1 file contents");
        assert_eq!(dest_app2_file_contents, "App 2 file contents");
        assert_eq!(dest_app1_nested_file_contents, "Nested 1 file contents");
    }
    match &actual[3].action_results[0] {
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
    set_all_platform_paths(
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

    let actual_resolved_path = actual[0].resolved_paths[0]
        .as_ref()
        .unwrap()
        .to_string_lossy();
    assert_eq!(actual_resolved_path.into_owned(), expected_resolved_path);
    match actual[0].action_results[0].as_ref() {
        Some(Err(TendrilActionError::IoError(e))) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        },
        Some(e) => assert_eq!(format!("{:?}", e), "dsfsd"),
        _ => panic!()
    }
}

// TODO: Test when the second tendril is a parent/child to the first tendril
