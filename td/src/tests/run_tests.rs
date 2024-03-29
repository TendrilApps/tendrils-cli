use crate::{ERR_PREFIX, run};
use crate::cli::{TendrilCliArgs, TendrilsSubcommands};
use crate::writer::Writer;
use tendrils::{ActionMode, is_tendrils_dir};
use tendrils::test_utils::{
    get_disposable_dir,
    is_empty,
    parse_tendrils_expose,
    set_parents,
    symlink_expose,
    Setup,
};
use rstest::rstest;
use serial_test::serial;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;
const TENDRILS_VAR_NAME: &str = "TENDRILS_FOLDER";

struct MockWriter {
    all_output: String,
}

impl MockWriter {
    fn new() -> MockWriter {
        MockWriter {
            all_output: "".to_string(),
        }
    }

    fn all_output_lines(&self) -> Vec<String> {
        self.all_output.lines().map( |x| String::from(x) ).collect()
    }
}

impl Writer for MockWriter {
    fn write(&mut self, text: &str) {
        self.all_output.push_str(text);
    }

    fn writeln(&mut self, text: &str) {
        self.all_output.push_str(text);
        self.all_output.push('\n');
    }
}

#[rstest]
#[case(true)]
#[case(false)]
#[serial("cd")]
fn init_no_path_given_uses_current_dir(#[case] force: bool) {
    let mut writer = MockWriter::new();
    let temp_dir = tempdir::TempDir::new_in(
        get_disposable_dir(),
        "InitFolder",
    ).unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: None,
        }
    };

    run(args, &mut writer);

    assert!(temp_dir.path().join("tendrils.json").exists());
    assert_eq!(writer.all_output, format!(
        "Created a Tendrils folder at: {}.\n",
        temp_dir.path().to_string_lossy()
    ));
}

#[rstest]
#[case(true)]
#[case(false)]
#[serial("cd")]
fn init_path_given_uses_given_path_and_ignores_valid_current_dir(#[case] force: bool) {
    let mut writer = MockWriter::new();
    let temp_dir = tempdir::TempDir::new_in(
        get_disposable_dir(),
        "TempDir",
    ).unwrap();
    let cd = temp_dir.path().join("CurrentDir");
    let given_dir = temp_dir.path().join("GivenDir");
    create_dir_all(&cd).unwrap();
    create_dir_all(&given_dir).unwrap();
    std::env::set_current_dir(&cd).unwrap();
    assert!(is_empty(&cd));

    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(given_dir.to_string_lossy().into()),
        }
    };

    run(args, &mut writer);

    assert!(given_dir.join("tendrils.json").exists());
    assert_eq!(writer.all_output, format!(
        "Created a Tendrils folder at: {}.\n",
        given_dir.to_string_lossy()
    ));
}

#[rstest]
#[case(true)]
#[case(false)]
#[serial("cd")]
fn init_path_given_uses_given_path_and_ignores_invalid_current_dir(#[case] force: bool) {
    let mut writer = MockWriter::new();
    let temp_dir = tempdir::TempDir::new_in(
        get_disposable_dir(),
        "TempDir",
    ).unwrap();
    let cd = temp_dir.path().join("CurrentDir");
    let given_dir = temp_dir.path().join("GivenDir");
    create_dir_all(&cd).unwrap();
    create_dir_all(&given_dir).unwrap();
    std::env::set_current_dir(&cd).unwrap();
    // Current dir is already a Tendrils folder
    // and has other misc files in it making it
    // invalid for init
    write(cd.join("tendrils.json"), "").unwrap();
    write(cd.join("misc.txt"), "").unwrap();
    create_dir_all(cd.join("misc")).unwrap();
    assert!(is_tendrils_dir(&cd));

    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(given_dir.to_string_lossy().into()),
        }
    };

    run(args, &mut writer);

    assert!(given_dir.join("tendrils.json").exists());
    assert_eq!(writer.all_output, format!(
        "Created a Tendrils folder at: {}.\n",
        given_dir.to_string_lossy()
    ));
}

#[rstest]
#[case(true)]
#[case(false)]
#[cfg_attr(windows, ignore)]
#[serial("cd")]
fn init_path_given_uses_given_path_and_ignores_missing_current_dir(#[case] force: bool) {
    let mut writer = MockWriter::new();
    let temp_dir = tempdir::TempDir::new_in(
        get_disposable_dir(),
        "TempDir",
    ).unwrap();
    let cd = temp_dir.path().join("CurrentDir");
    let given_dir = temp_dir.path().join("GivenDir");
    create_dir_all(&cd).unwrap();
    create_dir_all(&given_dir).unwrap();
    std::env::set_current_dir(&cd).unwrap();
    std::fs::remove_dir(&cd).unwrap();

    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(given_dir.to_string_lossy().into()),
        }
    };

    run(args, &mut writer);

    assert!(given_dir.join("tendrils.json").exists());
    assert_eq!(writer.all_output, format!(
        "Created a Tendrils folder at: {}.\n",
        given_dir.to_string_lossy()
    ));
}

#[rstest]
#[case(true)]
#[case(false)]
#[cfg_attr(windows, ignore)]
#[serial("cd")]
fn init_no_path_given_and_no_cd_prints_error_message(#[case] force: bool) {
    let mut writer = MockWriter::new();
    let temp_dir = tempdir::TempDir::new_in(
        get_disposable_dir(),
        "CurrentDir",
    ).unwrap();
    let cd = temp_dir.path();
    std::env::set_current_dir(&cd).unwrap();
    std::fs::remove_dir(&cd).unwrap();

    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: None,
        }
    };

    run(args, &mut writer);

    assert!(!cd.exists());
    assert_eq!(
        writer.all_output,
        format!("{ERR_PREFIX}: Could not get the current directory.\n")
    );
}

#[rstest]
#[case(true)]
#[case(false)]
fn init_non_empty_dir_prints_error_message_unless_forced(#[case] force: bool) {
    let mut writer = MockWriter::new();
    let temp_dir = tempdir::TempDir::new_in(
        get_disposable_dir(),
        "TempDir",
    ).unwrap();
    let given_dir = temp_dir.path();
    write(given_dir.join("misc.txt"), "").unwrap();

    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(given_dir.to_string_lossy().into()),
        }
    };

    run(args, &mut writer);

    if force {
        assert!(given_dir.join("tendrils.json").exists());
        assert_eq!(writer.all_output, format!(
            "Created a Tendrils folder at: {}.\n",
            given_dir.to_string_lossy()
        ));
    }
    else {
        assert!(!given_dir.join("tendrils.json").exists());
        let expected = format!("{ERR_PREFIX}: This folder is not empty. \
        Creating a Tendrils folder here may interfere with the existing \
        contents.\n\
        Consider running with the 'force' flag to ignore this error:\n\
        \n\
        td init --force\n");
        assert_eq!(writer.all_output, expected);
    }
}

#[rstest]
#[case(true)]
#[case(false)]
fn init_dir_is_already_tendrils_dir_prints_error_message(#[case] force: bool) {
    let mut writer = MockWriter::new();
    let temp_dir = tempdir::TempDir::new_in(
        get_disposable_dir(),
        "TempDir",
    ).unwrap();
    let given_dir = temp_dir.path();
    write(given_dir.join("tendrils.json"), "").unwrap();
    assert!(is_tendrils_dir(&given_dir));

    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(given_dir.to_string_lossy().into()),
        }
    };

    run(args, &mut writer);

    assert_eq!(
        writer.all_output, 
        format!("{ERR_PREFIX}: This folder is already a Tendrils folder.\n"),
    );
}

#[rstest]
#[case(true)]
#[case(false)]
fn init_dir_does_exists_prints_io_error_message(#[case] force: bool) {
    let mut writer = MockWriter::new();
    let given_dir = PathBuf::from("I do not exist");

    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Init {
            force,
            path: Some(given_dir.to_string_lossy().into()),
        }
    };

    run(args, &mut writer);

    assert_eq!(writer.all_output, format!("{ERR_PREFIX}: entity not found.\n"));
}

#[test]
#[serial("mut-env-var-td-folder")]
fn path_with_env_var_unset_prints_message() {
    let mut writer = MockWriter::new();
    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Path
    };
    std::env::remove_var(TENDRILS_VAR_NAME);
    let expected = format!(
        "The '{}' environment variable is not set.\n", TENDRILS_VAR_NAME
    );

    run(args, &mut writer);

    assert_eq!(writer.all_output, expected);
}

#[test]
#[serial("mut-env-var-td-folder")]
fn path_with_env_var_set_prints_path() {
    let mut writer = MockWriter::new();
    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Path
    };
    std::env::set_var(TENDRILS_VAR_NAME, "SomePath");

    // Formatted as hyperlink
    let expected = "\u{1b}]8;;SomePath\u{1b}\\SomePath\u{1b}]8;;\u{1b}\\\n";

    run(args, &mut writer);

    assert_eq!(writer.all_output, expected);
}

#[rstest]
#[serial("cd")]
#[cfg_attr(windows, ignore)]
fn tendril_action_no_path_given_and_no_cd_prints_message(
    #[values(ActionMode::Pull, ActionMode::Push, ActionMode::Link)]
    mode: ActionMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let delete_me = tempdir::TempDir::new_in(
        get_disposable_dir(),
        "DeleteMe"
    ).unwrap();
    std::env::set_current_dir(delete_me.path()).unwrap();
    std::fs::remove_dir_all(delete_me.path()).unwrap();

    let mut writer = MockWriter::new();
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {
            path: None, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
        ActionMode::Push => TendrilsSubcommands::Push {
            path: None, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
        ActionMode::Link => TendrilsSubcommands::Link {
            path: None, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
    };

    let args = TendrilCliArgs{
        tendrils_command,
    };
    let expected = format!(
        "{ERR_PREFIX}: Could not get the current directory.\n"
    );

    run(args, &mut writer);

    assert_eq!(writer.all_output, expected);
}

// TODO: Test no path given and cd is not tendrils folder
// TODO: Test no path given and cd is tendrils folder

#[rstest]
#[serial("cd")]
fn tendril_action_given_path_is_not_tendrils_dir_but_cd_is_should_print_message(
    #[values(ActionMode::Pull, ActionMode::Push, ActionMode::Link)]
    mode: ActionMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    let given_path = PathBuf::from("SomePathThatDoesn'tExist");
    setup.make_td_dir();
    write(&setup.td_json_file, "[]").unwrap();
    std::env::set_current_dir(&setup.td_dir).unwrap();

    let mut writer = MockWriter::new();
    let path = Some(given_path.to_str().unwrap().to_string());
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
        ActionMode::Push => TendrilsSubcommands::Push {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
        ActionMode::Link => TendrilsSubcommands::Link {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
    };

    let args = TendrilCliArgs{
        tendrils_command,
    };
    let expected = format!(
        "{ERR_PREFIX}: The given path is not a Tendrils folder.\n"
    );

    run(args, &mut writer);

    // To free the TempDir from use
    std::env::set_current_dir(setup.temp_dir.path().parent().unwrap()).unwrap();

    assert!(is_tendrils_dir(&setup.td_dir));
    assert!(!is_tendrils_dir(&given_path));
    assert_eq!(writer.all_output, expected);
}

#[rstest]
#[serial("cd")]
fn tendril_action_given_path_and_cd_are_both_tendrils_dirs_uses_given_path(
    #[values(ActionMode::Pull, ActionMode::Push, ActionMode::Link)]
    mode: ActionMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    let given_path = setup.parent_dir.join("GivenDir");
    setup.make_td_json_file(&[]);
    create_dir_all(&given_path).unwrap();
    write(given_path.join("tendrils.json"), "").unwrap();
    assert!(parse_tendrils_expose("[]").unwrap().is_empty());
    assert!(parse_tendrils_expose("").is_err());
    std::env::set_current_dir(&setup.td_dir).unwrap();

    let mut writer = MockWriter::new();
    let path = Some(given_path.to_str().unwrap().to_string());
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
        ActionMode::Push => TendrilsSubcommands::Push {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
        ActionMode::Link => TendrilsSubcommands::Link {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
    };
    let args = TendrilCliArgs{
        tendrils_command,
    };

    let expected = format!("{ERR_PREFIX}: Could not parse the tendrils.json \
    file.\nEOF while parsing a value at line 1 column 0\n");

    run(args, &mut writer);

    // To free the TempDir from use
    std::env::set_current_dir(setup.temp_dir.path().parent().unwrap()).unwrap();

    assert!(is_tendrils_dir(&setup.td_dir));
    assert!(is_tendrils_dir(&given_path));
    assert_eq!(writer.all_output, expected);
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
fn tendril_action_dry_run_does_not_modify(
    #[case] mode: ActionMode,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_local_file();
    setup.make_target_file();
    if mode == ActionMode::Link {
        // Setup remote file as symlink to some random (non-tendril) file
        symlink_expose(&setup.remote_file, &setup.target_file, false, false).unwrap();
    }
    else {
        setup.make_remote_file();
    }

    let mut tendril = setup.file_tendril_bundle();
    tendril.link = mode == ActionMode::Link;
    set_parents(&mut tendril, &[setup.parent_dir.clone()]);
    setup.make_td_json_file(&[tendril]);

    let mut writer = MockWriter::new();
    let path = Some(setup.td_dir.to_str().unwrap().to_string());
    let dry_run = true;
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
        ActionMode::Push => TendrilsSubcommands::Push {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
        ActionMode::Link => TendrilsSubcommands::Link {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
    };
    let args = TendrilCliArgs{
        tendrils_command,
    };

    run(args, &mut writer);

    if mode == ActionMode::Link {
        assert_eq!(setup.remote_file_contents(), "Target file contents");
    }
    else {
        assert_eq!(setup.remote_file_contents(), "Remote file contents");
    }
    assert_eq!(setup.local_file_contents(), "Local file contents");
    assert!(writer.all_output_lines()[3].contains("Skipped"));
    assert_eq!(setup.td_dir.read_dir().unwrap().into_iter().count(), 2);
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
fn tendril_action_tendrils_are_filtered_by_mode(
    #[case] mode: ActionMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
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

    setup.make_td_json_file(&[t1, t2, t3]);

    let mut writer = MockWriter::new();
    let path = Some(setup.td_dir.to_str().unwrap().to_string());
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
        ActionMode::Push => TendrilsSubcommands::Push {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
        ActionMode::Link => TendrilsSubcommands::Link {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: vec![]
        },
    };
    let args = TendrilCliArgs{
        tendrils_command,
    };

    run(args, &mut writer);

    if mode == ActionMode::Link {
        assert!(writer.all_output_lines()[3].contains("misc2.txt"));
        assert!(writer.all_output_lines()[3].contains("NotFound"));
        assert!(writer.all_output_lines()[5].contains("misc3.txt"));
        assert!(writer.all_output_lines()[5].contains("NotFound"));
        assert_eq!(writer.all_output_lines().len(), 7);
    }
    else {
        assert!(writer.all_output_lines()[3].contains("misc1.txt"));
        assert!(writer.all_output_lines()[3].contains("NotFound"));
        assert_eq!(writer.all_output_lines().len(), 5);
    }
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
fn tendril_action_tendrils_are_filtered_by_group(
    #[case] mode: ActionMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();

    let mut t1 = setup.file_tendril_bundle();
    let mut t2 = setup.file_tendril_bundle();
    let mut t3 = setup.file_tendril_bundle();
    t1.group = "App1".to_string();
    t2.group = "App2".to_string();
    t3.group = "App3".to_string();
    t1.link = mode == ActionMode::Link;
    t2.link = mode == ActionMode::Link;
    t3.link = mode == ActionMode::Link;
    set_parents(&mut t1, &[setup.parent_dir.clone()]);
    set_parents(&mut t2, &[setup.parent_dir.clone()]);
    set_parents(&mut t3, &[setup.parent_dir.clone()]);

    setup.make_td_json_file(&[t1, t2, t3]);

    let group_filters = vec!["App2".to_string(), "App3".to_string()];

    let mut writer = MockWriter::new();
    let path = Some(setup.td_dir.to_str().unwrap().to_string());
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {
            path, dry_run, force, groups: group_filters, names: vec![],
            profiles: vec![]
        },
        ActionMode::Push => TendrilsSubcommands::Push {
            path, dry_run, force, groups: group_filters, names: vec![],
            profiles: vec![]
        },
        ActionMode::Link => TendrilsSubcommands::Link {
            path, dry_run, force, groups: group_filters, names: vec![],
            profiles: vec![]
        },
    };
    let args = TendrilCliArgs{
        tendrils_command,
    };

    run(args, &mut writer);

    assert!(writer.all_output_lines()[3].contains("App2"));
    assert!(writer.all_output_lines()[3].contains("NotFound"));
    assert!(writer.all_output_lines()[5].contains("App3"));
    assert!(writer.all_output_lines()[5].contains("NotFound"));
    assert_eq!(writer.all_output_lines().len(), 7);
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
fn tendril_action_tendrils_are_filtered_by_names(
    #[case] mode: ActionMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
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
    set_parents(&mut t1, &[setup.parent_dir.clone()]);
    set_parents(&mut t2, &[setup.parent_dir.clone()]);
    set_parents(&mut t3, &[setup.parent_dir.clone()]);

    setup.make_td_json_file(&[t1, t2, t3]);

    let names_filter = vec!["misc2.txt".to_string(), "misc3.txt".to_string()];

    let mut writer = MockWriter::new();
    let path = Some(setup.td_dir.to_str().unwrap().to_string());
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {
            path, dry_run, force, groups: vec![], names: names_filter,
            profiles: vec![]
        },
        ActionMode::Push => TendrilsSubcommands::Push {
            path, dry_run, force, groups: vec![], names: names_filter,
            profiles: vec![]
        },
        ActionMode::Link => TendrilsSubcommands::Link {
            path, dry_run, force, groups: vec![], names: names_filter,
            profiles: vec![]
        },
    };
    let args = TendrilCliArgs{
        tendrils_command,
    };

    run(args, &mut writer);

    assert!(writer.all_output_lines()[3].contains("misc2.txt"));
    assert!(writer.all_output_lines()[3].contains("NotFound"));
    assert!(writer.all_output_lines()[5].contains("misc3.txt"));
    assert!(writer.all_output_lines()[5].contains("NotFound"));
    assert_eq!(writer.all_output_lines().len(), 7);
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
fn tendril_action_tendrils_are_filtered_by_profile(
    #[case] mode: ActionMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
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

    setup.make_td_json_file(&[t1, t2, t3]);

    let profiles_filter = vec!["p1".to_string(), "p2".to_string()];

    let mut writer = MockWriter::new();
    let path = Some(setup.td_dir.to_str().unwrap().to_string());
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: profiles_filter
        },
        ActionMode::Push => TendrilsSubcommands::Push {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: profiles_filter
        },
        ActionMode::Link => TendrilsSubcommands::Link {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: profiles_filter
        },
    };
    let args = TendrilCliArgs{
        tendrils_command,
    };

    run(args, &mut writer);

    assert!(writer.all_output_lines()[3].contains("misc2.txt"));
    assert!(writer.all_output_lines()[3].contains("NotFound"));
    assert!(writer.all_output_lines()[5].contains("misc3.txt"));
    assert!(writer.all_output_lines()[5].contains("NotFound"));
    assert_eq!(writer.all_output_lines().len(), 7);
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
fn tendril_action_empty_tendrils_array_should_print_message(
    #[case] mode: ActionMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    setup.make_td_json_file(&[]);

    let given_profiles = vec![];

    let mut writer = MockWriter::new();
    let path = Some(setup.td_dir.to_str().unwrap().to_string());
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: given_profiles
        },
        ActionMode::Push => TendrilsSubcommands::Push {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: given_profiles
        },
        ActionMode::Link => TendrilsSubcommands::Link {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: given_profiles
        },
    };
    let args = TendrilCliArgs{
        tendrils_command,
    };

    run(args, &mut writer);

    assert_eq!(writer.all_output, "No tendrils were found.\n");
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
#[case(ActionMode::Link)]
fn tendril_action_empty_filtered_tendrils_array_should_print_message(
    #[case] mode: ActionMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let setup = Setup::new();
    let mut t1 = setup.file_tendril_bundle();
    t1.names = vec!["misc1.txt".to_string()];
    t1.link = mode == ActionMode::Link;
    t1.profiles = vec!["ExcludeMe".to_string()];
    set_parents(&mut t1, &[setup.parent_dir.clone()]);
    setup.make_td_json_file(&[t1]);

    let given_profiles = vec!["NoMatch".to_string()];

    let mut writer = MockWriter::new();
    let path = Some(setup.td_dir.to_str().unwrap().to_string());
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: given_profiles
        },
        ActionMode::Push => TendrilsSubcommands::Push {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: given_profiles
        },
        ActionMode::Link => TendrilsSubcommands::Link {
            path, dry_run, force, groups: vec![], names: vec![],
            profiles: given_profiles
        },
    };
    let args = TendrilCliArgs{
        tendrils_command,
    };

    run(args, &mut writer);

    assert_eq!(writer.all_output, "No tendrils matched the given filter(s).\n");
}

// TODO: Test multiple_paths_only_copies_first for pull (see old commits in pull_tendril_tests)
// TODO: Test multiple_paths_first_is_missing_returns_not_found_error (see old commits in pull_tendril_tests)
// TODO: Test duplicate_tendrils_returns_duplicate_error_for_second_occurence_onward (see old pull_tests)
