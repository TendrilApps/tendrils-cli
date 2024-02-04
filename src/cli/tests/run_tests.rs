use crate::action_mode::ActionMode;
use crate::cli::{run, TendrilCliArgs, TendrilsSubcommands};
use crate::cli::writer::Writer;
use crate::{is_tendrils_dir, parse_tendrils, symlink};
use crate::tendril::Tendril;
use crate::test_utils::{get_disposable_dir, set_all_platform_paths};
use rstest::rstest;
use serial_test::serial;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;
use tempdir::TempDir;
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
    let expected = "SomePath\n";

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
    let delete_me = TempDir::new_in(
        get_disposable_dir(),
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
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();

    let current_dir = temp_parent_dir.path().join("CurrentDir");
    let given_dir = PathBuf::from("SomePathThatDoesn'tExist");
    create_dir_all(&current_dir).unwrap();
    write(&current_dir.join("tendrils.json"), "[]").unwrap();
    std::env::set_current_dir(&current_dir).unwrap();

    // TODO: Setup current directory as tendrils folder
    let mut writer = MockWriter::new();
    let path = Some(given_dir.to_str().unwrap().to_string());
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
    std::env::set_current_dir(temp_parent_dir.path().parent().unwrap()).unwrap();

    assert!(is_tendrils_dir(&current_dir));
    assert!(!is_tendrils_dir(&given_dir));
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
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();

    let current_dir = temp_parent_dir.path().join("CurrentDir");
    let given_dir = temp_parent_dir.path().join("GivenDir");
    create_dir_all(&current_dir).unwrap();
    create_dir_all(&given_dir).unwrap();
    write(current_dir.join("tendrils.json"), "[]").unwrap();
    write(given_dir.join("tendrils.json"), "").unwrap();
    assert!(parse_tendrils("[]").unwrap().is_empty());
    assert!(parse_tendrils("").is_err());
    std::env::set_current_dir(&current_dir).unwrap();

    let mut writer = MockWriter::new();
    let path = Some(given_dir.to_str().unwrap().to_string());
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
    std::env::set_current_dir(temp_parent_dir.path().parent().unwrap()).unwrap();

    assert!(is_tendrils_dir(&current_dir));
    assert!(is_tendrils_dir(&given_dir));
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
    let temp_parent_dir = TempDir::new_in(
        get_disposable_dir(),
        "ParentDir"
    ).unwrap();

    let mut tendril = Tendril::new("SomeApp", "misc.txt");
    tendril.link = mode == ActionMode::Link;

    set_all_platform_paths(&mut tendril, &[temp_parent_dir.path().to_path_buf()]);

    let tendrils_json = serde_json::to_string(&[tendril]).unwrap();
    let tendrils_dir = temp_parent_dir.path().join("TendrilsDir");
    let local_file = temp_parent_dir.path().join("misc.txt");
    let td_file = tendrils_dir.join("SomeApp").join("misc.txt");
    let target_file = temp_parent_dir.path().join("target.txt");
    create_dir_all(&td_file.parent().unwrap()).unwrap();
    write(tendrils_dir.join("tendrils.json"), tendrils_json).unwrap();
    write(&td_file, "Controlled file contents").unwrap();
    write(&target_file, "Orig target file contents").unwrap();
    if mode == ActionMode::Link {
        symlink(&local_file, &target_file, false, false).unwrap();
    }
    else {
        write(&local_file, "Local file contents").unwrap();
    }

    let mut writer = MockWriter::new();
    let path = Some(tendrils_dir.to_str().unwrap().to_string());
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

    let local_file_contents = read_to_string(local_file).unwrap();
    let td_file_contents = read_to_string(td_file).unwrap();
    if mode == ActionMode::Link {
        assert_eq!(local_file_contents, "Orig target file contents");
    }
    else {
        assert_eq!(local_file_contents, "Local file contents");
    }
    assert_eq!(td_file_contents, "Controlled file contents");
    assert_eq!(writer.all_output_lines()[0], "No local overrides were found.");
    assert!(writer.all_output_lines()[1].contains("Err(Skipped)"));
    assert_eq!(tendrils_dir.read_dir().unwrap().into_iter().count(), 2);
}

// TODO: Test uses_correct_platform_paths (see old commits in pull_tendril_tests)
// TODO: Test multiple_paths_only_copies_first for pull (see old commits in pull_tendril_tests)
// TODO: Test multiple_paths_first_is_missing_returns_not_found_error (see old commits in pull_tendril_tests)
// TODO: Test duplicate_tendrils_returns_duplicate_error_for_second_occurence_onward (see old pull_tests)
// TODO: Test that empty path list returns skipped error (for any action)
