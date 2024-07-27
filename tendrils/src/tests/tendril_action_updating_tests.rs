//! Tests that the updater function behaves properly. For additional
//! tests see the similar [`super::tendril_action_tests`] module

use crate::test_utils::Setup;
use crate::{
    ActionLog,
    ActionMode,
    FilterSpec,
    FsoType,
    TendrilActionSuccess,
    TendrilReport,
    TendrilsActor,
    TendrilsApi,
};
use rstest::rstest;
use std::rc::Rc;

#[rstest]
fn empty_tendrils_list_returns_empty(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_td_json_file(&[]);
    let mut actual = vec![];
    let updater = |r| actual.push(r);
    let filter = FilterSpec::new();

    api.tendril_action_updating(
        updater,
        mode,
        Some(&setup.td_repo),
        filter,
        dry_run,
        force)
        .unwrap();

    assert!(actual.is_empty());
    assert!(!setup.local_file.exists())
}

#[rstest]
fn returns_result_after_each_operation(
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_remote_file();
    setup.make_remote_nested_file();
    let mut bundle = setup.file_tendril_bundle();
    bundle.names.push("misc".to_string()); // Add the folder
    setup.make_td_json_file(&[bundle.clone()]);
    let filter = FilterSpec::new();

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

    api.tendril_action_updating(
        updater,
        ActionMode::Pull,
        Some(&setup.td_repo),
        filter,
        dry_run,
        force,
    )
    .unwrap();

    assert_eq!(call_counter, 2);
}
