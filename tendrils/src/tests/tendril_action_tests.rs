//! Tests that the action setup works properly.
//! See [`super::batch_tendril_action_tests`] for testing of the
//! core action functionality.

use crate::test_utils::Setup;
use crate::{
    ActionLog,
    ActionMode,
    FilterSpec,
    GetConfigError,
    GetTendrilsDirError,
    Location,
    SetupError,
    TendrilActionError,
    TendrilReport,
    TendrilsActor,
    TendrilsApi,
};
use rstest::rstest;
use serial_test::serial;
use std::fs::write;
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
    let filter = FilterSpec::new();

    let actual = api.tendril_action(
        mode,
        Some(&setup.td_dir),
        filter,
        dry_run,
        force
    )
    .unwrap();

    assert!(actual.is_empty());
    assert!(!setup.local_file.exists());
}

#[rstest]
fn empty_filtered_tendrils_list_returns_empty(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let tendril = setup.file_tendril_bundle();
    setup.make_td_json_file(&[tendril]);
    let mut filter = FilterSpec::new();
    let name_filter = ["I don't exist".to_string()];
    filter.names = &name_filter;

    let actual = api.tendril_action(
        mode,
        Some(&setup.td_dir),
        filter,
        dry_run,
        force
    )
    .unwrap();

    assert!(actual.is_empty());
    assert!(!setup.local_file.exists());
}

#[rstest]
fn given_td_dir_is_invalid_returns_no_valid_td_dir_err(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let filter = FilterSpec::new();
    assert!(!api.is_tendrils_dir(&setup.td_dir));

    let actual = api.tendril_action(
        mode,
        Some(&setup.td_dir),
        filter,
        dry_run,
        force
    );

    assert_eq!(
        actual,
        Err(SetupError::NoValidTendrilsDir(GetTendrilsDirError::GivenInvalid {
            path: setup.td_dir
        }))
    );
}

#[rstest]
#[serial("mut-env-var-td-folder")]
fn given_td_dir_is_none_global_td_dir_invalid_returns_no_valid_td_dir_err(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let filter = FilterSpec::new();
    assert!(!api.is_tendrils_dir(&setup.td_dir));
    std::env::set_var("TENDRILS_FOLDER", setup.td_dir.clone());

    let actual = api.tendril_action(
        mode,
        None,
        filter,
        dry_run,
        force
    );

    assert_eq!(
        actual,
        Err(SetupError::NoValidTendrilsDir(GetTendrilsDirError::GlobalInvalid {
            path: setup.td_dir
        }))
    );
}

#[rstest]
#[serial("mut-env-var-td-folder")]
fn given_td_dir_is_none_global_td_dir_not_set_returns_no_valid_td_dir_err(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let filter = FilterSpec::new();
    assert!(!api.is_tendrils_dir(&setup.td_dir));
    std::env::remove_var("TENDRILS_FOLDER");

    let actual = api.tendril_action(
        mode,
        None,
        filter,
        dry_run,
        force
    );

    assert_eq!(
        actual,
        Err(SetupError::NoValidTendrilsDir(GetTendrilsDirError::GlobalNotSet))
    );
}

#[rstest]
#[serial("mut-env-var-td-folder")]
fn given_td_dir_is_none_global_td_dir_is_valid_uses_global_td_dir(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let mut tendril = setup.file_tendril_bundle();
    tendril.link = mode == ActionMode::Link;
    setup.make_td_json_file(&[tendril.clone()]);
    std::env::set_var("TENDRILS_FOLDER", setup.td_dir.clone());
    let filter = FilterSpec::new();

    let actual = api.tendril_action(
        mode,
        None,
        filter,
        dry_run,
        force
    )
    .unwrap();

    assert_eq!(
        actual,
        vec![TendrilReport {
            orig_tendril: Rc::new(tendril.clone()),
            name: tendril.names[0].clone(),
            log: Ok(ActionLog::new(
                None,
                None,
                setup.remote_file,
                Err(TendrilActionError::IoError {
                    kind: std::io::ErrorKind::NotFound,
                    loc: Location::Source
                })
            ))
        }]
    );
}

#[rstest]
fn tendrils_json_invalid_returns_config_error(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let filter = FilterSpec::new();
    setup.make_td_dir();
    write(setup.td_json_file, "I'm not JSON").unwrap();

    let actual = api.tendril_action(
        mode,
        Some(&setup.td_dir),
        filter,
        dry_run,
        force
    );

    assert_eq!(
        actual,
        Err(SetupError::ConfigError(GetConfigError::ParseError(
            "expected value at line 1 column 1".to_string()
        )))
    );
}

#[rstest]
fn tendrils_are_filtered_before_action(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let mut tendril = setup.file_tendril_bundle();
    tendril.names.push("misc".to_string()); // Add folder
    tendril.link = mode == ActionMode::Link;
    setup.make_td_json_file(&[tendril.clone()]);
    let mut filter = FilterSpec::new();
    let name_filter = ["misc.txt".to_string()];
    filter.names = &name_filter;

    let actual = api.tendril_action(
        mode,
        Some(&setup.td_dir),
        filter,
        dry_run,
        force
    )
    .unwrap();

    tendril.names.pop(); // Expect that names are filtered from the bundle
    assert_eq!(
        actual,
        vec![TendrilReport {
            orig_tendril: Rc::new(tendril.clone()),
            name: "misc.txt".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                setup.remote_file,
                Err(TendrilActionError::IoError {
                    kind: std::io::ErrorKind::NotFound,
                    loc: Location::Source
                })
            ))
        }]
    );
}
