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
use std::path::MAIN_SEPARATOR_STR as SEP;
use tendrils::test_utils::{get_disposable_dir, MockTendrilsApi};
use tendrils::{
    ActionLog,
    ActionMode,
    ConfigType,
    FilterSpec,
    FsoType,
    GetConfigError,
    GetTendrilsRepoError,
    InitError,
    Location,
    RawTendril,
    SetupError,
    TendrilActionError,
    TendrilActionSuccess,
    TendrilMode,
    TendrilReport,
    TendrilsActor,
};

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
    locals: Vec<String>,
    remotes: Vec<String>,
    profiles: Vec<String>,
) -> TendrilsSubcommands {
    let action_args = ActionArgs { path, dry_run, force };
    let filter_args = FilterArgs { locals, remotes, profiles };

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
#[serial(SERIAL_CD)]
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
        format!("Created a Tendrils repo at: {}\n", cd.to_string_lossy())
    );

    // Cleanup
    std::env::set_current_dir(temp_dir.path().parent().unwrap()).unwrap();
}

#[rstest]
#[case(true)]
#[case(false)]
#[serial(SERIAL_CD)]
fn init_path_given_valid_uses_given_path_and_ignores_valid_current_dir(
    #[case] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let temp_dir =
        tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let cd = temp_dir.path().join("CurrentDir");
    let given_dir = PathBuf::from("/SomeGivenDir");
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
            "Created a Tendrils repo at: {}\n",
            &format!("{SEP}SomeGivenDir")
        )
    );

    // Cleanup
    std::env::set_current_dir(temp_dir.path().parent().unwrap()).unwrap();
}

#[rstest]
#[case(true)]
#[case(false)]
#[cfg_attr(windows, ignore)]
#[serial(SERIAL_CD)]
fn init_path_given_valid_uses_given_path_and_ignores_missing_current_dir(
    #[case] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let temp_dir =
        tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let cd = temp_dir.path().join("CurrentDir");
    let given_dir = PathBuf::from("/SomeGivenDir");
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
            "Created a Tendrils repo at: {}\n",
            given_dir.to_string_lossy()
        )
    );
}

#[rstest]
#[case(true)]
#[case(false)]
#[serial(SERIAL_CD)]
#[cfg_attr(windows, ignore)]
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
    let given_dir = PathBuf::from("/SomeGivenDir");

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
                "Created a Tendrils repo at: {SEP}SomeGivenDir\n"
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
fn init_dir_is_already_tendrils_repo_prints_error_message(#[case] force: bool) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let given_dir = PathBuf::from("/SomeGivenDir");

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
        format!("{ERR_PREFIX}: This folder is already a Tendrils repo\n"),
    );
}

#[rstest]
#[case(true)]
#[case(false)]
fn init_dir_does_not_exist_prints_io_error_message(#[case] force: bool) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let given_dir = PathBuf::from("/I do not exist");

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

#[rstest]
#[case(true)]
#[case(false)]
#[serial(SERIAL_CD)]
fn init_given_path_is_relative_prepends_with_cd(#[case] force: bool) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let temp_dir =
        tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let user_given_dir = PathBuf::from("../Relative/Path");
    let cd = temp_dir.path().join("CurrentDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();

    api.init_exp_dir_arg = cd.join(&user_given_dir);
    api.init_exp_force_arg = force;
    api.init_const_rt =
        Err(InitError::IoError { kind: std::io::ErrorKind::NotFound });

    let args = TendrilCliArgs {
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(user_given_dir.to_string_lossy().into()),
        },
    };

    let actual_exit_code = run(args, &api, &mut writer);

    // To free the TempDir from use
    std::env::set_current_dir(temp_dir.path().parent().unwrap()).unwrap();

    assert_eq!(actual_exit_code, Err(exitcode::IOERR));
}

#[rstest]
#[case(true)]
#[case(false)]
#[serial(SERIAL_CD)]
#[cfg_attr(windows, ignore)]
fn init_given_path_is_relative_and_cd_doesnt_exist_prepends_with_dir_sep(#[case] force: bool) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let temp_dir =
    tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let user_given_dir = "../Relative/Path";
    let cd = temp_dir.path().join("CurrentDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();
    std::fs::remove_dir(&cd).unwrap();

    api.init_exp_dir_arg = PathBuf::from("/../Relative/Path");
    api.init_exp_force_arg = force;
    api.init_const_rt =
        Err(InitError::IoError { kind: std::io::ErrorKind::NotFound });

    let args = TendrilCliArgs {
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(user_given_dir.to_string()),
        },
    };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Err(exitcode::IOERR));
    assert!(!cd.exists());
}

#[rstest]
#[case(true)]
#[case(false)]
#[serial(SERIAL_MUT_ENV_VARS)]
fn init_given_path_is_relative_but_resolves_to_abs_should_not_prepend_cd(#[case] force: bool) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    std::env::set_var("HOME", "/Some/Abs/Path");
    let user_given_dir = "~/Relative/Path";

    api.init_exp_dir_arg = PathBuf::from("/Some/Abs/Path/Relative/Path");
    api.init_exp_force_arg = force;
    api.init_const_rt =
        Err(InitError::IoError { kind: std::io::ErrorKind::NotFound });

    let args = TendrilCliArgs {
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(user_given_dir.to_string()),
        },
    };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Err(exitcode::IOERR));
}

#[rstest]
#[case(true)]
#[case(false)]
#[serial(SERIAL_CD, SERIAL_MUT_ENV_VARS)]
#[cfg_attr(windows, ignore)]
fn init_given_path_is_relative_but_resolves_to_abs_and_cd_doesnt_exist_should_not_prepend_cd(#[case] force: bool) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    std::env::set_var("HOME", "/Some/Abs/Path");
    let user_given_dir = "~/Relative/Path";
    let temp_dir =
        tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let cd = temp_dir.path().join("CurrentDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();
    std::fs::remove_dir(&cd).unwrap();

    api.init_exp_dir_arg = PathBuf::from("/Some/Abs/Path/Relative/Path");
    api.init_exp_force_arg = force;
    api.init_const_rt =
        Err(InitError::IoError { kind: std::io::ErrorKind::NotFound });

    let args = TendrilCliArgs {
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(user_given_dir.to_string()),
        },
    };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Err(exitcode::IOERR));
    assert!(!cd.exists());
}

#[test]
fn path_with_default_unset_prints_nothing() {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    api.get_default_repo_const_rt = Ok(None);
    let args = TendrilCliArgs { tendrils_command: TendrilsSubcommands::Path };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Ok(()));
    assert_eq!(writer.all_output, "");
}

#[test]
fn path_with_default_set_prints_path() {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    api.get_default_repo_const_rt = Ok(Some(PathBuf::from("SomePath")));
    let args = TendrilCliArgs {
        tendrils_command: TendrilsSubcommands::Path
    };

    // Formatted as hyperlink
    let expected = "\u{1b}]8;;SomePath\u{1b}\\SomePath\u{1b}]8;;\u{1b}\\\n";

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Ok(()));
    assert_eq!(writer.all_output, expected);
}

#[test]
fn path_io_error_accessing_global_config_file_prints_message() {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    api.get_default_repo_const_rt = Err(GetConfigError::IoError {
        cfg_type: ConfigType::Global,
        kind: std::io::ErrorKind::PermissionDenied,
    });
    let args = TendrilCliArgs { tendrils_command: TendrilsSubcommands::Path };

    let expected =
        format!("{ERR_PREFIX}: IO error while reading the global-config.json file:\n\
        permission denied\n");

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Err(exitcode::DATAERR));
    assert_eq!(writer.all_output, expected);
}

#[rstest]
#[serial(SERIAL_CD)]
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
    );
    let args = TendrilCliArgs { tendrils_command };

    let expected =
        format!("{ERR_PREFIX}: Could not get the current directory\n");

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Err(exitcode::OSERR));
    assert_eq!(writer.all_output, expected);
}

// TODO: Test no path given and cd is not Tendrils repo
// TODO: Test no path given and cd is Tendrils repo

#[rstest]
#[serial(SERIAL_CD)]
fn tendril_action_given_path_is_not_tendrils_repo_but_cd_is_should_print_message(
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
    let given_dir = PathBuf::from("/SomeGivenDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();

    api.is_tendrils_repo_fn = Some(Box::new(move |dir| {
        dir.inner() == cd_for_closure
    }));
    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&given_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Err(SetupError::NoValidTendrilsRepo(
        GetTendrilsRepoError::GivenInvalid { path: given_dir.clone() },
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
    );
    let args = TendrilCliArgs { tendrils_command };

    let expected =
        format!("{ERR_PREFIX}: /SomeGivenDir is not a Tendrils repo\n");

    let actual_exit_code = run(args, &api, &mut writer);

    // To free the TempDir from use
    std::env::set_current_dir(temp_dir.path().parent().unwrap()).unwrap();

    assert_eq!(actual_exit_code, Err(exitcode::NOINPUT));
    assert_eq!(writer.all_output, expected);
}

#[rstest]
#[serial(SERIAL_CD)]
fn tendril_action_given_path_and_cd_are_both_tendrils_repos_uses_given_path(
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
    let given_dir = PathBuf::from("/SomeGivenDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();

    api.is_tendrils_repo_const_rt = true;
    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&given_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Err(SetupError::ConfigError(GetConfigError::ParseError {
        cfg_type: ConfigType::Repo,
        msg: "Some parse error msg".to_string(),
    }));

    let path = Some(given_dir.to_str().unwrap().to_string());
    let tendrils_command = build_action_subcommand(
        path,
        mode,
        dry_run,
        force,
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
#[serial(SERIAL_CD)]
fn tendril_action_given_path_is_relative_prepends_with_cd(
    #[values(ActionMode::Pull, ActionMode::Push, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let temp_dir =
        tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let user_given_dir = "../Relative/Path";
    let cd = temp_dir.path().join("CurrentDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();
    let exp_passed_dir = cd.join(&user_given_dir);

    api.is_tendrils_repo_const_rt = true;
    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&exp_passed_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Err(SetupError::ConfigError(GetConfigError::ParseError {
        cfg_type: ConfigType::Repo,
        msg: "Some parse error msg".to_string(),
    }));

    let tendrils_command = build_action_subcommand(
        Some(user_given_dir.to_string()),
        mode,
        dry_run,
        force,
        vec![],
        vec![],
        vec![],
    );
    let args = TendrilCliArgs { tendrils_command };

    let actual_exit_code = run(args, &api, &mut writer);

    // To free the TempDir from use
    std::env::set_current_dir(temp_dir.path().parent().unwrap()).unwrap();

    assert_eq!(actual_exit_code, Err(exitcode::DATAERR));
}

#[rstest]
#[serial(SERIAL_CD)]
#[cfg_attr(windows, ignore)]
fn tendril_action_given_path_is_relative_and_cd_doesnt_exist_prepends_with_dir_sep(
    #[values(ActionMode::Pull, ActionMode::Push, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let temp_dir =
        tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let user_given_dir = "../Relative/Path";
    let cd = temp_dir.path().join("CurrentDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();
    std::fs::remove_dir(&cd).unwrap();
    let exp_passed_dir = PathBuf::from("/../Relative/Path");

    api.is_tendrils_repo_const_rt = true;
    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&exp_passed_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Err(SetupError::ConfigError(GetConfigError::ParseError {
        cfg_type: ConfigType::Repo,
        msg: "Some parse error msg".to_string(),
    }));

    let tendrils_command = build_action_subcommand(
        Some(user_given_dir.to_string()),
        mode,
        dry_run,
        force,
        vec![],
        vec![],
        vec![],
    );
    let args = TendrilCliArgs { tendrils_command };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Err(exitcode::DATAERR));
}

#[rstest]
#[serial(SERIAL_MUT_ENV_VARS)]
fn tendril_action_given_path_is_relative_but_resolves_to_abs_should_not_prepend_cd(
    #[values(ActionMode::Pull, ActionMode::Push, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    std::env::set_var("HOME", "/Some/Abs/Path");
    let user_given_dir = "~/Relative/Path";
    let exp_passed_dir = PathBuf::from("/Some/Abs/Path/Relative/Path");

    api.is_tendrils_repo_const_rt = true;
    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&exp_passed_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Err(SetupError::ConfigError(GetConfigError::ParseError {
        cfg_type: ConfigType::Repo,
        msg: "Some parse error msg".to_string(),
    }));

    let tendrils_command = build_action_subcommand(
        Some(user_given_dir.to_string()),
        mode,
        dry_run,
        force,
        vec![],
        vec![],
        vec![],
    );
    let args = TendrilCliArgs { tendrils_command };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Err(exitcode::DATAERR));
}

#[rstest]
#[serial(SERIAL_CD, SERIAL_MUT_ENV_VARS)]
#[cfg_attr(windows, ignore)]
fn tendril_action_given_path_is_relative_but_resolves_to_abs_and_cd_doesnt_exist_should_not_prepend_cd(
    #[values(ActionMode::Pull, ActionMode::Push, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    std::env::set_var("HOME", "/Some/Abs/Path");
    let user_given_dir = "~/Relative/Path";
    let exp_passed_dir = PathBuf::from("/Some/Abs/Path/Relative/Path");
    let temp_dir =
        tempdir::TempDir::new_in(get_disposable_dir(), "TempDir").unwrap();
    let cd = temp_dir.path().join("CurrentDir");
    create_dir_all(&cd).unwrap();
    std::env::set_current_dir(&cd).unwrap();
    std::fs::remove_dir(&cd).unwrap();

    api.is_tendrils_repo_const_rt = true;
    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&exp_passed_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Err(SetupError::ConfigError(GetConfigError::ParseError {
        cfg_type: ConfigType::Repo,
        msg: "Some parse error msg".to_string(),
    }));

    let tendrils_command = build_action_subcommand(
        Some(user_given_dir.to_string()),
        mode,
        dry_run,
        force,
        vec![],
        vec![],
        vec![],
    );
    let args = TendrilCliArgs { tendrils_command };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Err(exitcode::DATAERR));
}

#[rstest]
fn tendril_action_prints_returned_resolved_path_when_invalid_td_repo(
    #[values(ActionMode::Pull, ActionMode::Push, ActionMode::Link)]
    mode: ActionMode,
    #[values(true, false)] dry_run: bool,
    #[values(true, false)] force: bool,
) {
    let mut api = MockTendrilsApi::new();
    let mut writer = MockWriter::new();
    let given_dir = PathBuf::from("/Given/Path");

    api.is_tendrils_repo_const_rt = true;
    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&given_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Err(SetupError::NoValidTendrilsRepo(
        GetTendrilsRepoError::GivenInvalid {
            path: PathBuf::from("/Resolved/Returned/Path"),
        },
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
    );
    let args = TendrilCliArgs { tendrils_command };

    let actual_exit_code = run(args, &api, &mut writer);

    let expected =
        format!("{ERR_PREFIX}: /Resolved/Returned/Path is not a Tendrils repo\n");

    assert_eq!(actual_exit_code, Err(exitcode::NOINPUT));
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
    let given_dir = PathBuf::from("/SomeGivenDir");
    let mut t1 = RawTendril::new("SomeApp/misc.txt");
    let mut t2 = RawTendril::new("SomeApp/misc.txt");
    t1.remote = "r1/misc.txt".to_string();
    t2.remote = "r2/misc.txt".to_string();
    if mode == ActionMode::Link {
        t1.mode = TendrilMode::Link;
        t2.mode = TendrilMode::Link;
    }
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
            raw_tendril: t1.clone(),
            local: "SomeApp/misc.txt".to_string(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                PathBuf::from("r1/misc.txt"),
                ok_result,
            )),
        },
        TendrilReport {
            raw_tendril: t2.clone(),
            local: "SomeApp/misc.txt".to_string(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                PathBuf::from("r2/misc.txt"),
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
    );
    let args = TendrilCliArgs { tendrils_command };

    let _ = run(args, &api, &mut writer);

    // Update this example table as format changes in the future
    // ╭──────────────────┬─────────────┬──────────────────╮
    // │ Local            │ Remote      │ Report           │
    // ├──────────────────┼─────────────┼──────────────────┤
    // │ SomeApp/misc.txt │ r1/misc.txt │ Created          │
    // ├──────────────────┼─────────────┼──────────────────┤
    // │ SomeApp/misc.txt │ r2/misc.txt │ Source not found │
    // ╰──────────────────┴─────────────┴──────────────────╯
    let exp_link_n1 = ansi_hyperlink("r1/misc.txt", "r1/misc.txt");
    let exp_link_n2 = ansi_hyperlink("r2/misc.txt", "r2/misc.txt");
    let exp_output_lines = vec![
        "╭──────────────────┬─────────────┬──────────────────╮".to_string(),
        format!(
            "│ {color_bright_green}{style_underline}Local{color_reset}{style_reset}            \
            │ {color_bright_green}{style_underline}Remote{color_reset}{style_reset}      \
            │ {color_bright_green}{style_underline}Report{color_reset}{style_reset}           │"
        ),
        "├──────────────────┼─────────────┼──────────────────┤".to_string(),
        format!(
            "│ SomeApp/misc.txt │ {exp_link_n1} │ {color_bright_green}Created{color_reset}          │"
        ),
        "├──────────────────┼─────────────┼──────────────────┤".to_string(),
        format!(
            "│ SomeApp/misc.txt │ {exp_link_n2} │ {color_bright_red}Source not found{color_reset} │"
        ),
        "╰──────────────────┴─────────────┴──────────────────╯".to_string(),
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
    let given_dir = PathBuf::from("/SomeGivenDir");
    let mut t1 = RawTendril::new("SomeApp/misc.txt");
    let mut t2 = RawTendril::new("SomeApp/misc.txt");
    let mut t3 = RawTendril::new("SomeApp/misc.txt");
    t1.remote = "r1".to_string();
    t2.remote = "r2".to_string();
    t3.remote = "r3".to_string();
    if mode == ActionMode::Link {
        t1.mode = TendrilMode::Link;
        t2.mode = TendrilMode::Link;
        t3.mode = TendrilMode::Link;
    }
    let ok_result = Ok(TendrilActionSuccess::New);

    api.ta_exp_mode = mode.clone();
    api.ta_exp_path = Some(&given_dir);
    api.ta_exp_filter = FilterSpec::new();
    api.ta_exp_filter.mode = Some(mode.clone());
    api.ta_exp_dry_run = dry_run;
    api.ta_exp_force = force;
    api.ta_const_rt = Ok(vec![
        TendrilReport {
            raw_tendril: t1.clone(),
            local: "misc.txt".to_string(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                PathBuf::from("r1"),
                ok_result.clone(),
            )),
        },
        TendrilReport {
            raw_tendril: t2.clone(),
            local: "misc.txt".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("r2"),
                ok_result.clone(),
            )),
        },
        TendrilReport {
            raw_tendril: t3.clone(),
            local: "misc.txt".to_string(),
            log: Ok(ActionLog::new(
                Some(FsoType::Dir),
                None,
                PathBuf::from("r3"),
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
    let given_dir = PathBuf::from("/SomeGivenDir");
    let mut t1 = RawTendril::new("SomeApp/misc.txt");
    let mut t2 = RawTendril::new("SomeApp/misc.txt");
    let mut t3 = RawTendril::new("SomeApp/misc.txt");
    t1.remote = "r1".to_string();
    t2.remote = "r2".to_string();
    t3.remote = "r3".to_string();
    if mode == ActionMode::Link {
        t1.mode = TendrilMode::Link;
        t2.mode = TendrilMode::Link;
        t3.mode = TendrilMode::Link;
    }
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
            raw_tendril: t1.clone(),
            local: "misc.txt".to_string(),
            log: Ok(ActionLog::new(
                Some(FsoType::File),
                None,
                PathBuf::from("r1"),
                ok_result.clone(),
            )),
        },
        TendrilReport {
            raw_tendril: t2.clone(),
            local: "misc.txt".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("r2"),
                err_result,
            )),
        },
        TendrilReport {
            raw_tendril: t3.clone(),
            local: "misc.txt".to_string(),
            log: Ok(ActionLog::new(
                Some(FsoType::Dir),
                None,
                PathBuf::from("r3"),
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
    let given_dir = PathBuf::from("/SomeGivenDir");
    let mut t1_1 = RawTendril::new("l1");
    let mut t1_2 = RawTendril::new("l1");
    let mut t1_3 = RawTendril::new("l1");
    let mut t2_1 = RawTendril::new("l2");
    let mut t2_2 = RawTendril::new("l2");
    let mut t2_3 = RawTendril::new("l2");
    let mut t3_1 = RawTendril::new("l3");
    let mut t3_2 = RawTendril::new("l3");
    let mut t3_3 = RawTendril::new("l3");
    t1_1.remote = "r1_1".to_string();
    t1_2.remote = "r1_2".to_string();
    t1_3.remote = "r1_3".to_string();
    t2_1.remote = "r2_1".to_string();
    t2_2.remote = "r2_2".to_string();
    t2_3.remote = "r2_3".to_string();
    t3_1.remote = "r3_1".to_string();
    t3_2.remote = "r3_2".to_string();
    t3_3.remote = "r3_3".to_string();

    if mode == ActionMode::Link {
        t1_1.mode = TendrilMode::Link;
        t1_2.mode = TendrilMode::Link;
        t1_3.mode = TendrilMode::Link;
        t2_1.mode = TendrilMode::Link;
        t2_2.mode = TendrilMode::Link;
        t2_3.mode = TendrilMode::Link;
        t3_1.mode = TendrilMode::Link;
        t3_2.mode = TendrilMode::Link;
        t3_3.mode = TendrilMode::Link;
    }
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
            raw_tendril: t2_3.clone(),
            local: "l2".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("r2_3"),
                result.clone(),
            )),
        },
        TendrilReport {
            raw_tendril: t2_2.clone(),
            local: "l2".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("r2_2"),
                result.clone(),
            )),
        },
        TendrilReport {
            raw_tendril: t2_1.clone(),
            local: "l2".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("r2_1"),
                result.clone(),
            )),
        },
        TendrilReport {
            raw_tendril: t1_2.clone(),
            local: "l1".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("r1_2"),
                result.clone(),
            )),
        },
        TendrilReport {
            raw_tendril: t1_1.clone(),
            local: "l1".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("r1_1"),
                result.clone(),
            )),
        },
        TendrilReport {
            raw_tendril: t1_3.clone(),
            local: "l1".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("r1_3"),
                result.clone(),
            )),
        },
        TendrilReport {
            raw_tendril: t3_3.clone(),
            local: "l3".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("r3_3"),
                result.clone(),
            )),
        },
        TendrilReport {
            raw_tendril: t3_3.clone(),
            local: "l3".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("r3_1"),
                result.clone(),
            )),
        },
        TendrilReport {
            raw_tendril: t3_2.clone(),
            local: "l3".to_string(),
            log: Ok(ActionLog::new(
                None,
                None,
                PathBuf::from("r3_2"),
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
    assert!(writer.all_output_lines()[3] .contains("r2_3"));
    assert!(writer.all_output_lines()[5] .contains("r2_2"));
    assert!(writer.all_output_lines()[7] .contains("r2_1"));
    assert!(writer.all_output_lines()[9] .contains("r1_2"));
    assert!(writer.all_output_lines()[11].contains("r1_1"));
    assert!(writer.all_output_lines()[13].contains("r1_3"));
    assert!(writer.all_output_lines()[15].contains("r3_3"));
    assert!(writer.all_output_lines()[17].contains("r3_1"));
    assert!(writer.all_output_lines()[19].contains("r3_2"));
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
    let given_dir = PathBuf::from("/SomeGivenDir");
    let locals_filter = vec!["l1".to_string(), "l2".to_string()];
    let remotes_filter = vec!["r1".to_string(), "r2".to_string()];
    let profiles_filter = vec!["p1".to_string(), "p2".to_string()];
    let filter = FilterSpec {
        mode: Some(mode.clone()),
        locals: &locals_filter.clone(),
        remotes: &remotes_filter.clone(),
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
        locals_filter,
        remotes_filter,
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
    let given_dir = PathBuf::from("/SomeGivenDir");

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
    );
    let args = TendrilCliArgs { tendrils_command };

    let actual_exit_code = run(args, &api, &mut writer);

    assert_eq!(actual_exit_code, Ok(()));
    assert_eq!(writer.all_output, "No tendrils matched the given filter(s)\n");
}
