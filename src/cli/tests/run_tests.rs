use crate::action_mode::ActionMode;
use crate::cli::{run, TendrilCliArgs, TendrilsSubcommands};
use crate::cli::writer::Writer;
use crate::{is_tendrils_dir, parse_tendrils, symlink};
use crate::test_utils::{set_all_platform_paths, Setup};
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
#[cfg(not(windows))]
fn tendril_action_no_path_given_and_no_cd_prints_message(
    #[values(ActionMode::Pull, ActionMode::Push, ActionMode::Link)]
    mode: ActionMode,

    #[values(true, false)]
    dry_run: bool,

    #[values(true, false)]
    force: bool,
) {
    let delete_me = tempdir::TempDir::new_in(
        crate::test_utils::get_disposable_dir(),
        "DeleteMe"
    ).unwrap();
    std::env::set_current_dir(delete_me.path()).unwrap();
    std::fs::remove_dir_all(delete_me.path()).unwrap();

    let mut writer = MockWriter::new();
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {path: None, dry_run, force},
        ActionMode::Push => TendrilsSubcommands::Push {path: None, dry_run, force},
        ActionMode::Link => TendrilsSubcommands::Link {path: None, dry_run, force},
    };

    let args = TendrilCliArgs{
        tendrils_command,
    };
    let expected = "Error: Could not get the current directory\n";

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
        ActionMode::Pull => TendrilsSubcommands::Pull {path, dry_run, force},
        ActionMode::Push => TendrilsSubcommands::Push {path, dry_run, force},
        ActionMode::Link => TendrilsSubcommands::Link {path, dry_run, force},
    };

    let args = TendrilCliArgs{
        tendrils_command,
    };
    let expected = "Error: The given path is not a Tendrils folder\n";

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
    assert!(parse_tendrils("[]").unwrap().is_empty());
    assert!(parse_tendrils("").is_err());
    std::env::set_current_dir(&setup.td_dir).unwrap();

    let mut writer = MockWriter::new();
    let path = Some(given_path.to_str().unwrap().to_string());
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {path, dry_run, force},
        ActionMode::Push => TendrilsSubcommands::Push {path, dry_run, force},
        ActionMode::Link => TendrilsSubcommands::Link {path, dry_run, force},
    };
    let args = TendrilCliArgs{
        tendrils_command,
    };

    let expected = "Error: Could not parse the tendrils.json file\n";

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
    setup.make_ctrl_file();
    setup.make_target_file();
    if mode == ActionMode::Link {
        // Setup local file as symlink to some random (non-tendril) file
        symlink(&setup.local_file, &setup.target_file, false, false).unwrap();
    }
    else {
        setup.make_local_file();
    }

    let mut tendril = setup.file_tendril();
    tendril.link = mode == ActionMode::Link;
    set_all_platform_paths(&mut tendril, &[setup.parent_dir.clone()]);
    setup.make_td_json_file(&[tendril]);

    let mut writer = MockWriter::new();
    let path = Some(setup.td_dir.to_str().unwrap().to_string());
    let dry_run = true;
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {path, dry_run, force},
        ActionMode::Push => TendrilsSubcommands::Push {path, dry_run, force},
        ActionMode::Link => TendrilsSubcommands::Link {path, dry_run, force},
    };
    let args = TendrilCliArgs{
        tendrils_command,
    };

    run(args, &mut writer);

    if mode == ActionMode::Link {
        assert_eq!(setup.local_file_contents(), "Target file contents");
    }
    else {
        assert_eq!(setup.local_file_contents(), "Local file contents");
    }
    assert_eq!(setup.ctrl_file_contents(), "Controlled file contents");
    assert_eq!(writer.all_output_lines()[0], "No local overrides were found.");
    assert!(writer.all_output_lines()[4].contains("Skipped"));
    assert_eq!(setup.td_dir.read_dir().unwrap().into_iter().count(), 2);
}

// TODO: Test uses_correct_platform_paths (see old commits in pull_tendril_tests)
// TODO: Test multiple_paths_only_copies_first for pull (see old commits in pull_tendril_tests)
// TODO: Test multiple_paths_first_is_missing_returns_not_found_error (see old commits in pull_tendril_tests)
// TODO: Test duplicate_tendrils_returns_duplicate_error_for_second_occurence_onward (see old pull_tests)
// TODO: Test that empty path list returns skipped error (for any action)
