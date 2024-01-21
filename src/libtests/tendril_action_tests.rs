use crate::{tendril_action, TendrilActionError};
use crate::action_mode::ActionMode;
use crate::tendril::Tendril;
use crate::tendril_action_report::TendrilActionReport;
use crate::test_utils::{get_disposable_folder, is_empty, set_all_platform_paths};
use fs_extra::file::read_to_string;
use rstest::rstest;
use std::fs::{create_dir_all, write};
use tempdir::TempDir;

#[rstest]
fn given_empty_list_returns_empty(
    #[values(true, false)]
    dry_run: bool,
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
) {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let given_tendrils_folder = temp_parent_folder.path().join("TendrilsFolder");

    let actual = tendril_action(mode, &given_tendrils_folder, &[], dry_run);

    assert!(actual.is_empty());
    assert!(is_empty(&given_tendrils_folder))
}

#[rstest]
fn pull_returns_tendril_and_result_for_each_given(
    #[values(true, false)]
    dry_run: bool,
) {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();
    let given_tendrils_folder = temp_parent_folder.path().join("TendrilsFolder");
    let given_parent_folder = temp_parent_folder.path().to_path_buf();
    let source_app1_file = given_parent_folder.join("misc1.txt");
    let source_app2_file = given_parent_folder.join("misc2.txt");
    let source_app1_folder = given_parent_folder.join("App1 Folder");
    let nested_app1_file = source_app1_folder.join("nested1.txt");
    let dest_app1_file = given_tendrils_folder.join("App1").join("misc1.txt");
    let dest_app2_file = given_tendrils_folder.join("App2").join("misc2.txt");
    let dest_app1_nested= given_tendrils_folder.join("App1").join("App1 Folder").join("nested1.txt");
    create_dir_all(&source_app1_folder).unwrap();
    write(&source_app1_file, "App 1 file contents").unwrap();
    write(&source_app2_file, "App 2 file contents").unwrap();
    write(&nested_app1_file, "Nested 1 file contents").unwrap();

    let mut given = [
        Tendril::new("App1", "misc1.txt"),
        Tendril::new("App2", "misc2.txt"),
        Tendril::new("App1", "App1 Folder"),
        Tendril::new("App2", "I don't exist"),
    ];

    set_all_platform_paths(&mut given[0], &[given_parent_folder.clone()]);
    set_all_platform_paths(&mut given[1], &[given_parent_folder.clone()]);
    set_all_platform_paths(&mut given[2], &[given_parent_folder.clone()]);
    set_all_platform_paths(&mut given[3], &[given_parent_folder.clone()]);

    let io_not_found_err = std::io::Error::from(std::io::ErrorKind::NotFound);
    let expected: Vec<TendrilActionReport> = match dry_run {
        true => {
            vec![
                TendrilActionReport {
                    orig_tendril: &given[0],
                    resolved_paths: vec![Ok(source_app1_file)],
                    action_results: vec![Some(Err(TendrilActionError::Skipped))],
                },
                TendrilActionReport {
                    orig_tendril: &given[1],
                    resolved_paths: vec![Ok(source_app2_file)],
                    action_results: vec![Some(Err(TendrilActionError::Skipped))],
                },
                TendrilActionReport {
                    orig_tendril: &given[2],
                    resolved_paths: vec![Ok(source_app1_folder)],
                    action_results: vec![Some(Err(TendrilActionError::Skipped))],
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    resolved_paths: vec![Ok(given_parent_folder.join("I don't exist"))],
                    action_results: vec![Some(Err(TendrilActionError::IoError(io_not_found_err)))],
                },
            ]
        },
        false => {
            vec![
                TendrilActionReport {
                    orig_tendril: &given[0],
                    resolved_paths: vec![Ok(source_app1_file)],
                    action_results: vec![Some(Ok(()))],
                },
                TendrilActionReport {
                    orig_tendril: &given[1],
                    resolved_paths: vec![Ok(source_app2_file)],
                    action_results: vec![Some(Ok(()))],
                },
                TendrilActionReport {
                    orig_tendril: &given[2],
                    resolved_paths: vec![Ok(source_app1_folder)],
                    // action_results: vec![Some(Ok(()))],
                    action_results: vec![Some(Err(TendrilActionError::Skipped))],
                },
                TendrilActionReport {
                    orig_tendril: &given[3],
                    resolved_paths: vec![Ok(given_parent_folder.join("I don't exist"))],
                    action_results: vec![Some(Err(TendrilActionError::IoError(io_not_found_err)))],
                },
            ]
        }
    };

    let actual = tendril_action(ActionMode::Pull, &given_tendrils_folder, &given, dry_run);

    for (i, actual_report) in actual.iter().enumerate() {
        assert_eq!(actual_report.orig_tendril, expected[i].orig_tendril);
        assert_eq!(actual_report.resolved_paths, expected[i].resolved_paths);
    }

    // TendrilActionError is not equatable so must match manually
    if dry_run {
        assert!(matches!(actual[0].action_results[0], Some(Err(TendrilActionError::Skipped))));
        assert!(matches!(actual[1].action_results[0], Some(Err(TendrilActionError::Skipped))));
        assert!(matches!(actual[2].action_results[0], Some(Err(TendrilActionError::Skipped))));

        assert!(!dest_app1_file.exists());
        assert!(!dest_app2_file.exists());
        assert!(!dest_app1_nested.exists());
    }
    else {
        assert!(matches!(actual[0].action_results[0], Some(Ok(()))));
        assert!(matches!(actual[1].action_results[0], Some(Ok(()))));
        assert!(matches!(actual[2].action_results[0], Some(Ok(()))));

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

// TODO: Test when the second tendril is a parent/child to the first tendril
