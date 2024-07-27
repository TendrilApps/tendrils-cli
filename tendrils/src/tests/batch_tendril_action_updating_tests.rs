//! Tests that the updater function behaves properly, for additional
//! tests see the similar [`super::batch_tendril_action_tests`] module

use crate::test_utils::{get_disposable_dir, is_empty, Setup};
use crate::{
    batch_tendril_action_updating,
    ActionLog,
    ActionMode,
    FsoType,
    TendrilActionSuccess,
    TendrilReport,
};
use rstest::rstest;
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

    batch_tendril_action_updating(updater, mode, &given_td_repo, vec![], dry_run, force);

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

    batch_tendril_action_updating(
        updater,
        ActionMode::Pull,
        &setup.td_repo,
        vec![bundle.clone()],
        dry_run,
        force,
    );

    assert_eq!(call_counter, 2);
}
