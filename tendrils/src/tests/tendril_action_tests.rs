//! Tests that the action setup works properly.
//! See [`super::batch_tendril_action_tests`] for testing of the
//! core action functionality.

use crate::test_utils::{
    default_repo_path_as_json,
    global_cfg_file,
    Setup,
    set_parents,
    symlink_expose,
};
use crate::{
    ActionLog,
    ActionMode,
    ConfigType,
    FilterSpec,
    FsoType,
    GetConfigError,
    GetTendrilsRepoError,
    Location,
    SetupError,
    TendrilActionError,
    TendrilActionSuccess,
    TendrilReport,
    TendrilsActor,
    TendrilsApi,
};
use rstest::rstest;
use serial_test::serial;
use std::fs::write;
use std::path::{MAIN_SEPARATOR_STR as SEP, PathBuf};
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
        Some(&setup.td_repo),
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
        Some(&setup.td_repo),
        filter,
        dry_run,
        force
    )
    .unwrap();

    assert!(actual.is_empty());
    assert!(!setup.local_file.exists());
}

#[rstest]
fn given_td_repo_is_invalid_returns_no_valid_td_repo_err(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let filter = FilterSpec::new();
    assert!(!api.is_tendrils_repo(&setup.td_repo));

    let actual = api.tendril_action(
        mode,
        Some(&setup.td_repo),
        filter,
        dry_run,
        force
    );

    assert_eq!(
        actual,
        Err(SetupError::NoValidTendrilsRepo(GetTendrilsRepoError::GivenInvalid {
            path: setup.td_repo
        }))
    );
}

#[rstest]
#[serial("mut-env-var-testing")]
fn given_td_repo_is_none_default_td_repo_invalid_returns_no_valid_td_repo_err(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let filter = FilterSpec::new();
    setup.make_global_cfg_file(
        default_repo_path_as_json("I DON'T EXIST"),
    );
    assert!(!api.is_tendrils_repo(&setup.td_repo));

    let actual = api.tendril_action(
        mode,
        None,
        filter,
        dry_run,
        force
    );

    assert_eq!(
        actual,
        Err(SetupError::NoValidTendrilsRepo(GetTendrilsRepoError::DefaultInvalid {
            path: PathBuf::from(SEP).join("I DON'T EXIST")
        }))
    );
}

#[rstest]
#[serial("mut-env-var-testing")]
fn given_td_repo_is_none_default_td_repo_not_set_returns_no_valid_td_repo_err(
    #[values(ActionMode::Push, ActionMode::Pull, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    let filter = FilterSpec::new();
    setup.set_home_dir();
    assert!(!global_cfg_file().exists());
    assert!(!api.is_tendrils_repo(&setup.td_repo));

    let actual = api.tendril_action(
        mode,
        None,
        filter,
        dry_run,
        force
    );

    assert_eq!(
        actual,
        Err(SetupError::NoValidTendrilsRepo(GetTendrilsRepoError::DefaultNotSet))
    );
}

#[rstest]
#[serial("mut-env-var-testing")]
fn given_td_repo_is_none_default_td_repo_is_valid_uses_default_td_repo(
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
    let json_path = setup.td_repo.to_string_lossy().replace("\\", "\\\\");
    setup.make_global_cfg_file(
        default_repo_path_as_json(&json_path),
    );
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
#[serial("mut-env-var-testing")]
fn leading_tilde_or_env_vars_in_given_repo_path_is_resolved(
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
    let exp_local_type;
    let exp_remote_type;
    if &mode == &ActionMode::Pull {
        setup.make_remote_file();
        exp_local_type = None;
        exp_remote_type = Some(FsoType::File);
    }
    else {
        setup.make_local_file();
        exp_local_type = Some(FsoType::File);
        exp_remote_type = None;
    }
    let filter = FilterSpec::new();
    setup.set_home_dir();
    std::env::set_var("var", "TendrilsRepo");
    let given_path = PathBuf::from("~/<var>");

    let actual = api.tendril_action(
        mode.clone(),
        Some(&given_path),
        filter,
        dry_run,
        force
    )
    .unwrap();

    let exp_success;
    if dry_run {
        exp_success = TendrilActionSuccess::NewSkipped;
    }
    else {
        exp_success = TendrilActionSuccess::New;

        if mode == ActionMode::Pull {
            assert_eq!(
                setup.local_file_contents(),
                "Remote file contents",
            );
        }
        else {
            assert_eq!(
                setup.remote_file_contents(),
                "Local file contents"
            );
        }
    }
    assert_eq!(
        actual,
        vec![TendrilReport {
            orig_tendril: Rc::new(tendril.clone()),
            name: tendril.names[0].clone(),
            log: Ok(ActionLog::new(
                exp_local_type,
                exp_remote_type,
                setup.remote_file.clone(),
                Ok(exp_success),
            ))
        }]
    );
}

#[rstest]
#[serial("mut-env-var-testing")]
fn leading_tilde_or_env_vars_in_default_repo_path_is_resolved(
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
    let exp_local_type;
    let exp_remote_type;
    if &mode == &ActionMode::Pull {
        setup.make_remote_file();
        exp_local_type = None;
        exp_remote_type = Some(FsoType::File);
    }
    else {
        setup.make_local_file();
        exp_local_type = Some(FsoType::File);
        exp_remote_type = None;
    }
    let filter = FilterSpec::new();
    setup.make_global_cfg_file(
        default_repo_path_as_json("~/<var>"),
    );
    std::env::set_var("var", "TendrilsRepo");

    let actual = api.tendril_action(
        mode.clone(),
        None,
        filter,
        dry_run,
        force
    )
    .unwrap();

    let exp_success;
    if dry_run {
        exp_success = TendrilActionSuccess::NewSkipped;
    }
    else {
        exp_success = TendrilActionSuccess::New;

        if mode == ActionMode::Pull {
            assert_eq!(
                setup.local_file_contents(),
                "Remote file contents",
            );
        }
        else {
            assert_eq!(
                setup.remote_file_contents(),
                "Local file contents"
            );
        }
    }
    assert_eq!(
        actual,
        vec![TendrilReport {
            orig_tendril: Rc::new(tendril.clone()),
            name: tendril.names[0].clone(),
            log: Ok(ActionLog::new(
                exp_local_type,
                exp_remote_type,
                setup.remote_file.clone(),
                Ok(exp_success),
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
    setup.make_dot_td_dir();
    write(setup.td_json_file, "I'm not JSON").unwrap();

    let actual = api.tendril_action(
        mode,
        Some(&setup.td_repo),
        filter,
        dry_run,
        force
    );

    assert_eq!(
        actual,
        Err(SetupError::ConfigError(GetConfigError::ParseError {
            cfg_type: ConfigType::Repo,
            msg: "expected value at line 1 column 1".to_string(),
        })),
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
        Some(&setup.td_repo),
        filter,
        dry_run,
        force
    )
    .unwrap();

    tendril.names.pop(); // Expect that names are filtered from the bundle
    assert_eq!(
        actual,
        vec![TendrilReport {
            orig_tendril: Rc::new(tendril),
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

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
fn tendril_action_dry_run_does_not_modify(
    #[case] mode: ActionMode,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_target_file();
    let filter = FilterSpec::new();
    if mode == ActionMode::Link {
        // Setup remote file as symlink to some random (non-tendril) file
        symlink_expose(&setup.remote_file, &setup.target_file, false, false)
            .unwrap();
    }
    else {
        setup.make_remote_file();
    }

    let mut tendril = setup.file_tendril_bundle();
    tendril.link = mode == ActionMode::Link;
    set_parents(&mut tendril, &[setup.parent_dir.clone()]);
    setup.make_td_json_file(&[tendril]);

    let dry_run = true;

    api.tendril_action(
        mode.clone(),
        Some(&setup.td_repo),
        filter,
        dry_run,
        force
    )
    .unwrap();

    if mode == ActionMode::Link {
        assert_eq!(setup.remote_file_contents(), "Target file contents");
    }
    else {
        assert_eq!(setup.remote_file_contents(), "Remote file contents");
    }
    assert_eq!(setup.local_file_contents(), "Local file contents");
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
#[case(ActionMode::Out)]
fn tendril_action_tendrils_are_filtered_by_mode(
    #[case] mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();

    let mut t1 = setup.file_tendril_bundle();
    let mut t2 = setup.file_tendril_bundle();
    let mut t3 = setup.file_tendril_bundle();
    t1.names = vec!["misc1.txt".to_string()];
    t2.names = vec!["misc2.txt".to_string()];
    t3.names = vec!["misc3.txt".to_string()];
    t1.link = false;
    t2.link = true;
    t3.link = true;
    set_parents(&mut t1, &[setup.parent_dir.clone()]);
    set_parents(&mut t2, &[setup.parent_dir.clone()]);
    set_parents(&mut t3, &[setup.parent_dir.clone()]);
    let io_err = Err(TendrilActionError::IoError {
        kind: std::io::ErrorKind::NotFound,
        loc: Location::Source
    });
    let t1_result = TendrilReport {
        orig_tendril: Rc::new(t1.clone()),
        name: "misc1.txt".to_string(),
        log: Ok(ActionLog::new(
            None,
            None,
            setup.parent_dir.join("misc1.txt"),
            io_err.clone(),
        )),
    };
    let t2_result = TendrilReport {
        orig_tendril: Rc::new(t2.clone()),
        name: "misc2.txt".to_string(),
        log: Ok(ActionLog::new(
            None,
            None,
            setup.parent_dir.join("misc2.txt"),
            io_err.clone(),
        )),
    };
    let t3_result = TendrilReport {
        orig_tendril: Rc::new(t3.clone()),
        name: "misc3.txt".to_string(),
        log: Ok(ActionLog::new(
            None,
            None,
            setup.parent_dir.join("misc3.txt"),
            io_err.clone(),
        )),
    };

    setup.make_td_json_file(&[t1.clone(), t2.clone(), t3.clone()]);
    let mut filter = FilterSpec::new();
    filter.mode = Some(mode.clone());

    let actual = api.tendril_action(
        mode.clone(),
        Some(&setup.td_repo),
        filter,
        dry_run,
        force
    )
    .unwrap();

    if mode == ActionMode::Link {
        assert_eq!(actual, vec![t2_result, t3_result]);
    }
    else if mode == ActionMode::Out {
        assert_eq!(actual, vec![t1_result, t2_result, t3_result]);
    }
    else {
        assert_eq!(actual, vec![t1_result]);
    }
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
#[case(ActionMode::Out)]
fn tendril_action_tendrils_are_filtered_by_group(
    #[case] mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();

    let mut t1 = setup.file_tendril_bundle();
    let mut t2 = setup.file_tendril_bundle();
    let mut t3 = setup.file_tendril_bundle();
    t1.names = vec!["misc1.txt".to_string()];
    t2.names = vec!["misc2.txt".to_string()];
    t3.names = vec!["misc3.txt".to_string()];
    t1.group = "App1".to_string();
    t2.group = "App2".to_string();
    t3.group = "App3".to_string();
    t1.link = mode == ActionMode::Link;
    t2.link = mode == ActionMode::Link;
    t3.link = mode == ActionMode::Link;
    set_parents(&mut t1, &[setup.parent_dir.clone()]);
    set_parents(&mut t2, &[setup.parent_dir.clone()]);
    set_parents(&mut t3, &[setup.parent_dir.clone()]);
    let io_err = Err(TendrilActionError::IoError {
        kind: std::io::ErrorKind::NotFound,
        loc: Location::Source
    });
    let t2_result = TendrilReport {
        orig_tendril: Rc::new(t2.clone()),
        name: "misc2.txt".to_string(),
        log: Ok(ActionLog::new(
            None,
            None,
            setup.parent_dir.join("misc2.txt"),
            io_err.clone(),
        )),
    };
    let t3_result = TendrilReport {
        orig_tendril: Rc::new(t3.clone()),
        name: "misc3.txt".to_string(),
        log: Ok(ActionLog::new(
            None,
            None,
            setup.parent_dir.join("misc3.txt"),
            io_err.clone(),
        )),
    };

    setup.make_td_json_file(&[t1.clone(), t2.clone(), t3.clone()]);
    let groups_filter = vec!["App2".to_string(), "App3".to_string()];
    let mut filter = FilterSpec::new();
    filter.groups = &groups_filter;

    let actual = api.tendril_action(
        mode.clone(),
        Some(&setup.td_repo),
        filter,
        dry_run,
        force
    )
    .unwrap();

    assert_eq!(actual, vec![t2_result, t3_result]);
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
#[case(ActionMode::Out)]
fn tendril_action_tendrils_are_filtered_by_parents(
    #[case] mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();

    let mut t1 = setup.file_tendril_bundle();
    let mut t2 = setup.file_tendril_bundle();
    let mut t3 = setup.file_tendril_bundle();
    t1.names = vec!["misc1.txt".to_string()];
    t2.names = vec!["misc2.txt".to_string()];
    t3.names = vec!["misc3.txt".to_string()];
    t1.parents = vec!["p/1".to_string()];
    t2.parents = vec!["p/2".to_string()];
    t3.parents = vec!["p/3".to_string()];
    t1.link = mode == ActionMode::Link;
    t2.link = mode == ActionMode::Link;
    t3.link = mode == ActionMode::Link;

    let loc;
    if mode == ActionMode::Pull {
        loc = Location::Source;
    }
    else {
        loc = Location::Dest;
    }
    let io_err = Err(TendrilActionError::IoError {
        kind: std::io::ErrorKind::NotFound,
        loc,
    });
    let t2_result = TendrilReport {
        orig_tendril: Rc::new(t2.clone()),
        name: "misc2.txt".to_string(),
        log: Ok(ActionLog::new(
            None,
            None,
            PathBuf::from(SEP).join("p").join("2").join("misc2.txt"),
            io_err.clone(),
        )),
    };
    let t3_result = TendrilReport {
        orig_tendril: Rc::new(t3.clone()),
        name: "misc3.txt".to_string(),
        log: Ok(ActionLog::new(
            None,
            None,
            PathBuf::from(SEP).join("p").join("3").join("misc3.txt"),
            io_err.clone(),
        )),
    };

    setup.make_td_json_file(&[t1.clone(), t2.clone(), t3.clone()]);
    let parents_filter = vec!["p/2".to_string(), "p/3".to_string()];
    let mut filter = FilterSpec::new();
    filter.parents = &parents_filter;

    let actual = api.tendril_action(
        mode.clone(),
        Some(&setup.td_repo),
        filter,
        dry_run,
        force
    )
    .unwrap();

    assert_eq!(actual, vec![t2_result, t3_result]);
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
#[case(ActionMode::Out)]
fn tendril_action_tendrils_are_filtered_by_profile(
    #[case] mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let api = TendrilsActor {};
    let setup = Setup::new();

    let mut t1 = setup.file_tendril_bundle();
    let mut t2 = setup.file_tendril_bundle();
    let mut t3 = setup.file_tendril_bundle();
    t1.names = vec!["misc1.txt".to_string()];
    t2.names = vec!["misc2.txt".to_string()];
    t3.names = vec!["misc3.txt".to_string()];
    t1.link = mode == ActionMode::Link;
    t2.link = mode == ActionMode::Link;
    t3.link = mode == ActionMode::Link;
    t1.profiles = vec!["ExcludeMe".to_string()];
    t2.profiles = vec!["p1".to_string()];
    t3.profiles = vec![];
    set_parents(&mut t1, &[setup.parent_dir.clone()]);
    set_parents(&mut t2, &[setup.parent_dir.clone()]);
    set_parents(&mut t3, &[setup.parent_dir.clone()]);
    let io_err = Err(TendrilActionError::IoError {
        kind: std::io::ErrorKind::NotFound,
        loc: Location::Source
    });
    let t2_result = TendrilReport {
        orig_tendril: Rc::new(t2.clone()),
        name: "misc2.txt".to_string(),
        log: Ok(ActionLog::new(
            None,
            None,
            setup.parent_dir.join("misc2.txt"),
            io_err.clone(),
        )),
    };
    let t3_result = TendrilReport {
        orig_tendril: Rc::new(t3.clone()),
        name: "misc3.txt".to_string(),
        log: Ok(ActionLog::new(
            None,
            None,
            setup.parent_dir.join("misc3.txt"),
            io_err.clone(),
        )),
    };

    setup.make_td_json_file(&[t1.clone(), t2.clone(), t3.clone()]);
    let profiles_filter = vec!["p1".to_string(), "p2".to_string()];
    let mut filter = FilterSpec::new();
    filter.profiles = &profiles_filter;

    let actual = api.tendril_action(
        mode.clone(),
        Some(&setup.td_repo),
        filter,
        dry_run,
        force
    )
    .unwrap();

    assert_eq!(actual, vec![t2_result, t3_result]);
}
