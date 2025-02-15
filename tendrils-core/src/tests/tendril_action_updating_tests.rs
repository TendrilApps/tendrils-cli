//! Tests that the updater function behaves properly. For additional
//! tests see the similar [`super::tendril_action_tests`] module

use crate::test_utils::Setup;
use crate::{
    ActionLog,
    ActionMode,
    CallbackUpdater,
    FilterSpec,
    FsoType,
    TendrilActionSuccess,
    TendrilReport,
    TendrilsActor,
    TendrilsApi,
};
use rstest::rstest;

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
    let filter = FilterSpec::new();

    let mut count_actual = -1;
    let mut before_actual = vec![];
    let mut after_actual = vec![];
    let count_fn = |c| count_actual = c;
    let before_fn = |raw| before_actual.push(raw);
    let after_fn = |report| after_actual.push(report);
    let updater =
        CallbackUpdater::<_, _, _, ActionLog>::new(count_fn, before_fn, after_fn);

    api.tendril_action_updating(
        updater,
        mode,
        Some(&setup.uni_td_repo()),
        filter,
        dry_run,
        force)
        .unwrap();

    assert_eq!(count_actual, 0);
    assert!(before_actual.is_empty());
    assert!(after_actual.is_empty());
    assert!(!setup.local_file.exists())
}

#[rstest]
fn returns_result_after_each_operation(
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_local_nested_file();
    let t1 = setup.file_tendril_raw();
    let mut t2 = t1.clone();
    t2.remote = setup.remote_nested_file.to_string_lossy().to_string();
    setup.make_td_json_file(&[t1.clone(), t2.clone()]);
    let filter = FilterSpec::new();

    let expected_success = match dry_run {
        true => Ok(TendrilActionSuccess::NewSkipped),
        false => Ok(TendrilActionSuccess::New),
    };

    let mut count_actual = -1;
    let mut before_actual = vec![];
    let mut after_actual = vec![];
    let mut after_call_counter = 0;
    let mut before_call_counter = 0;
    let count_fn = |c| count_actual = c;
    let before_fn = |raw| {
        before_call_counter += 1;
        if before_call_counter == 1 {
            assert_eq!(raw, t1);
        }
        else {
            assert_eq!(raw, t2);
        }
        before_actual.push(raw);
    };
    let after_fn = |report| {
        after_call_counter += 1;
        if after_call_counter == 1 {
            assert_eq!(report, TendrilReport {
                raw_tendril: t1.clone(),
                local: t1.local.clone(),
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
        else if after_call_counter == 2 {
            assert_eq!(report, TendrilReport {
                raw_tendril: t2.clone(),
                local: t2.local.clone(),
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

        after_actual.push(report);
    };
    let updater =
        CallbackUpdater::<_, _, _, ActionLog>::new(count_fn, before_fn, after_fn);

    api.tendril_action_updating(
        updater,
        ActionMode::Push,
        Some(&setup.uni_td_repo()),
        filter,
        dry_run,
        force,
    )
    .unwrap();
}
