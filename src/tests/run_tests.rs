use crate::cli::{
    ansi_hyperlink,
    AboutSubcommands,
    ActionArgs,
    FilterArgs,
    TendrilCliArgs,
    TendrilsSubcommands,
};
use crate::{run, Writer, ERR_PREFIX};
use inline_colorization::{
    color_bright_green,
    color_bright_red,
    color_reset,
    style_reset,
    style_underline,
};
use rstest::rstest;
use serial_test::serial;
use std::fs::create_dir_all;
use std::path::PathBuf;
use tendrils::test_utils::{get_disposable_dir, MockTendrilsApi};
use tendrils::{
    ActionLog,
    ActionMode,
    FilterSpec,
    FsoType,
    GetConfigError,
    GetTendrilsDirError,
    InitError,
    Location,
    SetupError,
    TendrilActionError,
    TendrilActionSuccess,
    TendrilBundle,
    TendrilReport,
    TendrilsActor,
};

const TENDRILS_VAR_NAME: &str = "TENDRILS_FOLDER";

struct MockWriter {
    all_output: String,
}

impl MockWriter {
    fn new() -> MockWriter {
        MockWriter { all_output: "".to_string() }
    }

    fn all_output_lines(&self) -> Vec<String> {
        self.all_output.lines().map(String::from).collect()
    }
}

fn build_action_subcommand(
    path: Option<String>,
    mode: ActionMode,
    dry_run: bool,
    force: bool,
    groups: Vec<String>,
    names: Vec<String>,
    parents: Vec<String>,
    profiles: Vec<String>,
) -> TendrilsSubcommands {
    let action_args = ActionArgs { path, dry_run, force };
    let filter_args = FilterArgs { groups, names, parents, profiles };

    match mode {
        ActionMode::Pull => {
            TendrilsSubcommands::Pull { action_args, filter_args }
        }
        ActionMode::Push => {
            TendrilsSubcommands::Push { action_args, filter_args }
        }
        ActionMode::Link => {
            TendrilsSubcommands::Link { action_args, filter_args }
        }
        ActionMode::Out => {
            TendrilsSubcommands::Out { action_args, filter_args }
        }
    }
}

impl Writer for MockWriter {
    fn writeln(&mut self, text: &str) {
        self.all_output.push_str(text);
        self.all_output.push('\n');
    }
}

#[test]
fn about_license_returns_license_type_and_hyperlink_to_repo_license_file() {
    let api = TendrilsActor {};
    let mut writer = MockWriter::new();
    let version = env!["CARGO_PKG_VERSION"];
    let mut url = format![
        "https://github.com/TendrilApps/tendrils-cli/blob/{version}/LICENSE.md"
    ];
    url = ansi_hyperlink(&url, &url);
    let expected = format![
        "td is licensed under a GPL-3.0-or-later license.\n\nThe license text \
         is here:\n{url}\n"
    ];

    let args = TendrilCliArgs {
        tendrils_command: TendrilsSubcommands::About {
            about_subcommand: AboutSubcommands::License,
        },
    };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(writer.all_output, expected);
    assert_eq!(actual_exit_code, Ok(()));
}

#[test]
fn about_acknowledgements_returns_message_and_hyperlink_to_repo_third_party_file(
) {
    let api = TendrilsActor {};
    let mut writer = MockWriter::new();
    let version = env!["CARGO_PKG_VERSION"];
    let mut url = format![
        "https://github.com/TendrilApps/tendrils-cli/blob/{version}/LICENSE-3RD-PARTY.md"
    ];
    url = ansi_hyperlink(&url, &url);
    let expected = format![
        "td uses several open source dependencies.\n\nTheir acknowledgements \
         and licensing information are here:\n{url}\n"
    ];

    let args = TendrilCliArgs {
        tendrils_command: TendrilsSubcommands::About {
            about_subcommand: AboutSubcommands::Acknowledgements,
        },
    };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(writer.all_output, expected);
    assert_eq!(actual_exit_code, Ok(()));
}

#[rstest]
#[case(true)]
#[case(false)]
#[serial("cd")]
fn init_no_path_given_uses_current_dir(#[case] force: bool) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let temp_dir =
        tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let cd = temp_dir.path().join("CurrentDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();

    api.init_exp_dir_arg = cd.clone();
    api.init_exp_force_arg = force;
    api.init_const_rt = Ok(());

    let args = TendrilCliArgs {
        tendrils_command: TendrilsSubcommands::Init { force, path: None },
    };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Ok(()));
    assert_eq!(
        writer.all_output,
        format!("Created a Tendrils folder at: {}\n", cd.to_string_lossy())
    );

    // Cleanup
    std::env::set_current_dir(temp_dir.path().parent().unwrap()).unwrap();
}

#[rstest]
#[case(true)]
#[case(false)]
#[serial("cd")]
fn init_path_given_valid_uses_given_path_and_ignores_valid_current_dir(
    #[case] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let temp_dir =
        tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let cd = temp_dir.path().join("CurrentDir");
    let given_dir = PathBuf::from("SomeGivenDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();

    api.init_exp_dir_arg = given_dir.clone();
    api.init_exp_force_arg = force;
    api.init_const_rt = Ok(());

    let args = TendrilCliArgs {
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(given_dir.to_string_lossy().into()),
        },
    };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Ok(()));
    assert_eq!(
        writer.all_output,
        format!(
            "Created a Tendrils folder at: {}\n",
            given_dir.to_string_lossy()
        )
    );

    // Cleanup
    std::env::set_current_dir(temp_dir.path().parent().unwrap()).unwrap();
}

#[rstest]
#[case(true)]
#[case(false)]
#[cfg_attr(windows, ignore)]
#[serial("cd")]
fn init_path_given_valid_uses_given_path_and_ignores_missing_current_dir(
    #[case] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let temp_dir =
        tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let cd = temp_dir.path().join("CurrentDir");
    let given_dir = PathBuf::from("SomeGivenDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();
    std::fs::remove_dir(&cd).unwrap();

    api.init_exp_dir_arg = given_dir.to_path_buf();
    api.init_exp_force_arg = force;
    api.init_const_rt = Ok(());

    let args = TendrilCliArgs {
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(given_dir.to_string_lossy().into()),
        },
    };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Ok(()));
    assert_eq!(
        writer.all_output,
        format!(
            "Created a Tendrils folder at: {}\n",
            given_dir.to_string_lossy()
        )
    );
}

#[rstest]
#[case(true)]
#[case(false)]
#[cfg_attr(windows, ignore)]
#[serial("cd")]
fn init_no_path_given_and_no_cd_prints_error_message(#[case] force: bool) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let temp_dir =
        tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let cd = temp_dir.path().join("CurrentDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();
    std::fs::remove_dir(&cd).unwrap();

    api.init_exp_dir_arg = cd.to_path_buf();
    api.init_exp_force_arg = force;
    api.init_fn = Some(Box::new(|_, _| panic!()));

    let args = TendrilCliArgs {
        tendrils_command: TendrilsSubcommands::Init { force, path: None },
    };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Err(exitcode::OSERR));
    assert!(!cd.exists());
    assert_eq!(
        writer.all_output,
        format!("{ERR_PREFIX}: Could not get the current directory\n")
    );
}

#[rstest]
#[case(true)]
#[case(false)]
fn init_non_empty_dir_prints_error_message_unless_forced(#[case] force: bool) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let given_dir = PathBuf::from("SomeGivenDir");

    api.init_exp_dir_arg = given_dir.to_path_buf();
    api.init_exp_force_arg = force;
    api.init_fn = Some(Box::new(|_, force| match force {
        true => Ok(()),
        false => Err(InitError::NotEmpty),
    }));

    let args = TendrilCliArgs {
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(given_dir.to_string_lossy().into()),
        },
    };

    let actual_exit_code = run(args, &api, &mut writer);

    if force {
        assert_eq!(actual_exit_code, Ok(()));
        assert_eq!(
            writer.all_output,
            format!(
                "Created a Tendrils folder at: {}\n",
                given_dir.to_string_lossy()
            )
        );
    }
    else {
        assert_eq!(actual_exit_code, Err(exitcode::DATAERR));
        let expected = format!(
            "{ERR_PREFIX}: This folder is not empty. Creating a Tendrils \
             folder here may interfere with the existing contents.\nConsider \
             running with the 'force' flag to ignore this error:\n\ntd init \
             --force\n"
        );
        assert_eq!(writer.all_output, expected);
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn init_dir_is_already_tendrils_dir_prints_error_message(#[case] force: bool) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let given_dir = PathBuf::from("SomeGivenDir");

    api.init_exp_dir_arg = given_dir.to_path_buf();
    api.init_exp_force_arg = force;
    api.init_const_rt = Err(InitError::AlreadyInitialized);

    let args = TendrilCliArgs {
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(given_dir.to_string_lossy().into()),
        },
    };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Err(exitcode::DATAERR));
    assert_eq!(
        writer.all_output,
        format!("{ERR_PREFIX}: This folder is already a Tendrils folder\n"),
    );
}

#[rstest]
#[case(true)]
#[case(false)]
fn init_dir_does_not_exist_prints_io_error_message(#[case] force: bool) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let given_dir = PathBuf::from("I do not exist");

    api.init_exp_dir_arg = given_dir.to_path_buf();
    api.init_exp_force_arg = force;
    api.init_const_rt =
        Err(InitError::IoError { kind: std::io::ErrorKind::NotFound });

    let args = TendrilCliArgs {
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(given_dir.to_string_lossy().into()),
        },
    };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Err(exitcode::IOERR));
    assert_eq!(
        writer.all_output,
        format!("{ERR_PREFIX}: IO error - entity not found\n")
    );
}

#[test]
#[serial("mut-env-var-td-folder")]
fn path_with_env_var_unset_prints_message() {
    let api = TendrilsActor {};
    let mut writer = MockWriter::new();
    let args = TendrilCliArgs { tendrils_command: TendrilsSubcommands::Path };
    std::env::remove_var(TENDRILS_VAR_NAME);
    let expected = format!(
        "The '{}' environment variable is not set\n",
        TENDRILS_VAR_NAME
    );

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Ok(()));
    assert_eq!(writer.all_output, expected);
}

#[test]
#[serial("mut-env-var-td-folder")]
fn path_with_env_var_set_prints_path() {
    let api = TendrilsActor {};
    let mut writer = MockWriter::new();
    let args = TendrilCliArgs { tendrils_command: TendrilsSubcommands::Path };
    std::env::set_var(TENDRILS_VAR_NAME, "SomePath");

    // Formatted as hyperlink
    let expected = "\u{1b}]8;;SomePath\u{1b}\\SomePath\u{1b}]8;;\u{1b}\\\n";

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Ok(()));
    assert_eq!(writer.all_output, expected);
}

#[rstest]
#[serial("cd")]
#[cfg_attr(windows, ignore)]
fn tendril_action_no_path_given_and_no_cd_prints_message(
    #[values(ActionMode::Pull, ActionMode::Push, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let temp_dir =
        tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let cd = temp_dir.path().join("CurrentDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();
    std::fs::remove_dir(&cd).unwrap();

    api.ta_fn = Some(Box::new(|_, _, _, _, _| panic!()));

    let tendrils_command = build_action_subcommand(
        None,
        mode,
        dry_run,
        force,
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let args = TendrilCliArgs { tendrils_command };

    let expected =
        format!("{ERR_PREFIX}: Could not get the current directory\n");

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Err(exitcode::OSERR));
    assert_eq!(writer.all_output, expected);
}

// TODO: Test no path given and cd is not tendrils folder
// TODO: Test no path given and cd is tendrils folder

#[rstest]
#[serial("cd")]
fn tendril_action_given_path_is_not_tendrils_dir_but_cd_is_should_print_message(
    #[values(ActionMode::Pull, ActionMode::Push, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let temp_dir =
        tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let cd = temp_dir.path().join("CurrentDir");
    let cd_for_closure = cd.clone();
    let given_dir = PathBuf::from("SomeGivenDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();

    api.is_tendrils_dir_fn = Some(Box::new(move |dir| {
        if dir == cd_for_closure {
            true
        }
        else {
            false
        }
    }));
    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&given_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Err(SetupError::NoValidTendrilsDir(
        GetTendrilsDirError::GivenInvalid { path: given_dir.clone() },
    ));

    let path = Some(given_dir.to_str().unwrap().to_string());
    let tendrils_command = build_action_subcommand(
        path,
        mode,
        dry_run,
        force,
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let args = TendrilCliArgs { tendrils_command };

    let expected =
        format!("{ERR_PREFIX}: SomeGivenDir is not a Tendrils folder\n");

    let actual_exit_code = run(args, &api, &mut writer);

    // To free the TempDir from use
    std::env::set_current_dir(temp_dir.path().parent().unwrap()).unwrap();

    assert_eq!(actual_exit_code, Err(exitcode::NOINPUT));
    assert_eq!(writer.all_output, expected);
}

#[rstest]
#[serial("cd")]
fn tendril_action_given_path_and_cd_are_both_tendrils_dirs_uses_given_path(
    #[values(ActionMode::Pull, ActionMode::Push, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let temp_dir =
        tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let cd = temp_dir.path().join("CurrentDir");
    let given_dir = PathBuf::from("SomeGivenDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();

    api.is_tendrils_dir_const_rt = true;
    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&given_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Err(SetupError::ConfigError(GetConfigError::ParseError(
        "Some parse error msg".to_string(),
    )));

    let path = Some(given_dir.to_str().unwrap().to_string());
    let tendrils_command = build_action_subcommand(
        path,
        mode,
        dry_run,
        force,
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let args = TendrilCliArgs { tendrils_command };

    let expected = format!(
        "{ERR_PREFIX}: Could not parse the tendrils.json file:\nSome parse \
         error msg\n"
    );

    let actual_exit_code = run(args, &api, &mut writer);

    // To free the TempDir from use
    std::env::set_current_dir(temp_dir.path().parent().unwrap()).unwrap();

    assert_eq!(actual_exit_code, Err(exitcode::DATAERR));
    assert_eq!(writer.all_output, expected);
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
#[case(ActionMode::Out)]
fn tendril_action_prints_table_in_specific_format(
    #[case] mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let given_dir = PathBuf::from("SomeGivenDir");
    let link = mode == ActionMode::Link;
    let orig_tendril = std::rc::Rc::new(TendrilBundle {
        group: "SomeApp".to_string(),
        names: vec!["n1".to_string(), "n2".to_string()],
        parents: vec!["p1".to_string()],
        profiles: vec![],
        link,
        dir_merge: false,
    });
    let ok_result = Ok(TendrilActionSuccess::New);
    let err_result = Err(TendrilActionError::IoError {
        kind: std::io::ErrorKind::NotFound,
        loc: Location::Source,
    });

    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&given_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Ok(vec![
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n1".to_string(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                PathBuf::from("p1n1"),
                ok_result,
            )),
        },
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n2".to_string(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                PathBuf::from("p1n2"),
                err_result,
            )),
        },
    ]);

    let mut writer = MockWriter::new();
    let path = Some(given_dir.to_str().unwrap().to_string());
    let tendrils_command = build_action_subcommand(
        path,
        mode,
        dry_run,
        force,
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let args = TendrilCliArgs { tendrils_command };

    let _ = run(args, &api, &mut writer);

    // Update this example table as format changes in the future
    // ╭─────────┬──────┬──────┬──────────────────╮
    // │ Group   │ Name │ Path │ Report           │
    // ├─────────┼──────┼──────┼──────────────────┤
    // │ SomeApp │ n1   │ p1n1 │ Created          │
    // ├─────────┼──────┼──────┼──────────────────┤
    // │ SomeApp │ n2   │ p1n2 │ Source not found │
    // ╰─────────┴──────┴──────┴──────────────────╯
    let exp_link_n1 = ansi_hyperlink("p1n1", "p1n1");
    let exp_link_n2 = ansi_hyperlink("p1n2", "p1n2");
    let exp_output_lines = vec![
        "╭─────────┬──────┬──────┬──────────────────╮".to_string(),
        format!(
            "│ {color_bright_green}{style_underline}Group{color_reset}{style_reset}   \
            │ {color_bright_green}{style_underline}Name{color_reset}{style_reset} \
            │ {color_bright_green}{style_underline}Path{color_reset}{style_reset} \
            │ {color_bright_green}{style_underline}Report{color_reset}{style_reset}           │"
        ),
        "├─────────┼──────┼──────┼──────────────────┤".to_string(),
        format!(
            "│ SomeApp │ n1   │ {exp_link_n1} │ {color_bright_green}Created{color_reset}          │"
        ),
        "├─────────┼──────┼──────┼──────────────────┤".to_string(),
        format!(
            "│ SomeApp │ n2   │ {exp_link_n2} │ {color_bright_red}Source not found{color_reset} │"
        ),
        "╰─────────┴──────┴──────┴──────────────────╯".to_string(),
    ];

    for (i, exp_line) in exp_output_lines.into_iter().enumerate() {
        assert_eq!(
            writer.all_output_lines()[i],
            exp_line,
            "Failed on line: {i}"
        );
    }
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
#[case(ActionMode::Out)]
fn tendril_action_if_all_pass_they_are_totalled_and_returns_ok(
    #[case] mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let given_dir = PathBuf::from("SomeGivenDir");
    let link = mode == ActionMode::Link;
    let orig_tendril = std::rc::Rc::new(TendrilBundle {
        group: "SomeApp".to_string(),
        names: vec!["n1".to_string(), "n2".to_string(), "n3".to_string()],
        parents: vec!["p1".to_string()],
        profiles: vec![],
        link,
        dir_merge: false,
    });
    let ok_result = Ok(TendrilActionSuccess::New);

    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&given_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Ok(vec![
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n1".to_string(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                PathBuf::from("p1n1"),
                ok_result.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n2".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("p1n2"),
                ok_result.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n3".to_string(),
            log: Ok(ActionLog::new(
                Some(FsoType::Dir),
                None,
                PathBuf::from("p1n3"),
                ok_result,
            )),
        },
    ]);

    let mut writer = MockWriter::new();
    let path = Some(given_dir.to_str().unwrap().to_string());
    let tendrils_command = build_action_subcommand(
        path,
        mode,
        dry_run,
        force,
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let args = TendrilCliArgs { tendrils_command };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Ok(()));
    assert_eq!(
        writer.all_output_lines().last().unwrap().to_string(),
        format!(
            "Total: 3, Successful: {color_bright_green}3{color_reset}, \
             Failed: {color_bright_red}0{color_reset}"
        )
    );
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
#[case(ActionMode::Out)]
fn tendril_action_if_any_fail_they_are_totalled_and_returns_exit_code(
    #[case] mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let given_dir = PathBuf::from("SomeGivenDir");
    let link = mode == ActionMode::Link;
    let orig_tendril = std::rc::Rc::new(TendrilBundle {
        group: "SomeApp".to_string(),
        names: vec!["n1".to_string(), "n2".to_string(), "n3".to_string()],
        parents: vec!["p1".to_string()],
        profiles: vec![],
        link,
        dir_merge: false,
    });
    let ok_result = Ok(TendrilActionSuccess::New);
    let err_result = Err(TendrilActionError::IoError {
        kind: std::io::ErrorKind::NotFound,
        loc: Location::Source,
    });

    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&given_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Ok(vec![
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n1".to_string(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                PathBuf::from("p1n1"),
                ok_result.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n2".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("p1n2"),
                err_result,
            )),
        },
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n3".to_string(),
            log: Ok(ActionLog::new(
                Some(FsoType::Dir),
                None,
                PathBuf::from("p1n3"),
                ok_result,
            )),
        },
    ]);

    let mut writer = MockWriter::new();
    let path = Some(given_dir.to_str().unwrap().to_string());
    let tendrils_command = build_action_subcommand(
        path,
        mode,
        dry_run,
        force,
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let args = TendrilCliArgs { tendrils_command };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Err(exitcode::SOFTWARE));
    assert_eq!(
        writer.all_output_lines().last().unwrap().to_string(),
        format!(
            "Total: 3, Successful: {color_bright_green}2{color_reset}, \
             Failed: {color_bright_red}1{color_reset}"
        )
    );
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
#[case(ActionMode::Out)]
fn tendril_action_order_of_reports_is_unchanged(
    #[case] mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let given_dir = PathBuf::from("SomeGivenDir");
    let link = mode == ActionMode::Link;
    let orig_tendril = std::rc::Rc::new(TendrilBundle {
        group: "SomeApp".to_string(),
        // Non alphabetical order
        names: vec!["n2".to_string(), "n1".to_string(), "n3".to_string()],
        // Non alphabetical order
        parents: vec!["p3".to_string(), "p1".to_string(), "p2".to_string()],
        profiles: vec![],
        link,
        dir_merge: false,
    });
    let result = Err(TendrilActionError::IoError {
        kind: std::io::ErrorKind::NotFound,
        loc: Location::Source,
    });

    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&given_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Ok(vec![
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n2".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("p3n2"),
                result.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n2".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("p1n2"),
                result.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n2".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("p2n2"),
                result.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n1".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("p3n1"),
                result.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n1".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("p1n1"),
                result.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n1".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("p2n1"),
                result.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n3".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("p3n3"),
                result.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n3".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("p1n3"),
                result.clone(),
            )),
        },
        TendrilReport {
            orig_tendril: orig_tendril.clone(),
            name: "n3".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("p2n3"),
                result.clone(),
            )),
        },
    ]);

    let mut writer = MockWriter::new();
    let path = Some(given_dir.to_str().unwrap().to_string());
    let tendrils_command = build_action_subcommand(
        path,
        mode,
        dry_run,
        force,
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let args = TendrilCliArgs { tendrils_command };

    let actual_exit_code = run(args, &api, &mut writer);

    println!("{}", writer.all_output);
    assert_eq!(actual_exit_code, Err(exitcode::SOFTWARE));
    assert_eq!(
        writer.all_output_lines().last().unwrap().to_string(),
        format!(
            "Total: 9, Successful: {color_bright_green}0{color_reset}, \
             Failed: {color_bright_red}9{color_reset}"
        )
    );
    assert!(writer.all_output_lines()[3].contains("p3n2"));
    assert!(writer.all_output_lines()[5].contains("p1n2"));
    assert!(writer.all_output_lines()[7].contains("p2n2"));
    assert!(writer.all_output_lines()[9].contains("p3n1"));
    assert!(writer.all_output_lines()[11].contains("p1n1"));
    assert!(writer.all_output_lines()[13].contains("p2n1"));
    assert!(writer.all_output_lines()[15].contains("p3n3"));
    assert!(writer.all_output_lines()[17].contains("p1n3"));
    assert!(writer.all_output_lines()[19].contains("p2n3"));
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
#[case(ActionMode::Out)]
fn tendril_action_filters_are_passed_properly(
    #[case] mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let given_dir = PathBuf::from("SomeGivenDir");
    let groups_filter = vec!["App1".to_string(), "App2".to_string()];
    let names_filter = vec!["misc1.txt".to_string(), "misc2.txt".to_string()];
    let parents_filter = vec!["p/1".to_string(), "p/2".to_string()];
    let profiles_filter = vec!["P/1".to_string(), "P/2".to_string()];
    let filter = FilterSpec {
        mode: Some(mode.clone()),
        groups: &groups_filter.clone(),
        names: &names_filter.clone(),
        parents: &parents_filter.clone(),
        profiles: &profiles_filter.clone(),
    };

    // These assertions occur in the mock run call
    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&given_dir);
    api.ta_exp_filter = filter;
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Ok(vec![]);

    let mut writer = MockWriter::new();
    let path = Some(given_dir.to_str().unwrap().to_string());
    let tendrils_command = build_action_subcommand(
        path,
        mode.clone(),
        dry_run,
        force,
        groups_filter,
        names_filter,
        parents_filter,
        profiles_filter,
    );
    let args = TendrilCliArgs { tendrils_command };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Ok(()));
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
#[case(ActionMode::Out)]
fn tendril_action_empty_reports_list_prints_message(
    #[case] mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let given_dir = PathBuf::from("SomeGivenDir");

    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&given_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Ok(vec![]);

    let mut writer = MockWriter::new();
    let path = Some(given_dir.to_str().unwrap().to_string());
    let tendrils_command = build_action_subcommand(
        path,
        mode,
        dry_run,
        force,
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let args = TendrilCliArgs { tendrils_command };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Ok(()));
    assert_eq!(writer.all_output, "No tendrils matched the given filter(s)\n");
}
