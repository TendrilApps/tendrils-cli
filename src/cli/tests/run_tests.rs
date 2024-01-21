use crate::action_mode::ActionMode;
use crate::cli::{run, TendrilCliArgs, TendrilsSubcommands};
use crate::cli::writer::Writer;
use crate::{is_tendrils_folder, parse_tendrils};
use crate::tendril::Tendril;
use crate::test_utils::{get_disposable_folder, set_all_platform_paths};
use rstest::rstest;
use serial_test::serial;
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
}

impl Writer for MockWriter {
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
#[case(true)]
#[case(false)]
#[serial("cd")]
#[cfg(not(windows))]
fn tendril_action_no_path_given_and_no_cd_prints_message(#[case] dry_run: bool) {
    // TODO: Test with Push mode
    unimplemented!();
    let delete_me = TempDir::new_in(
        get_disposable_folder(),
        "DeleteMe"
    ).unwrap();
    std::env::set_current_dir(delete_me.path()).unwrap();
    std::fs::remove_dir(delete_me.path()).unwrap();

    let mut writer = MockWriter::new();
    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Pull {
            path: None,
            dry_run,
        }
    };
    let expected = "Error: Could not get the current directory\n";

    run(args, &mut writer);

    assert_eq!(writer.all_output, expected);
}

// TODO: Test no path given and cd is not tendrils folder
// TODO: Test no path given and cd is tendrils folder

#[rstest]
#[serial("cd")]
fn tendril_action_given_path_is_not_tendrils_folder_cd_is_prints_message(
    #[values(ActionMode::Pull, ActionMode::Push)]
    mode: ActionMode,

    #[values(true, false)]
    dry_run: bool,
) {
    // TODO: Setup current directory as tendrils folder
    let mut writer = MockWriter::new();
    let path = Some("SomePathThatDoesn'tExist".to_string());
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {path, dry_run},
        ActionMode::Push => TendrilsSubcommands::Push {path, dry_run},
        _ => unimplemented!(),
    };

    let args = TendrilCliArgs{
        tendrils_command,
    };
    let expected = "Error: The given path is not a Tendrils folder\n";

    run(args, &mut writer);

    assert_eq!(writer.all_output, expected);
}

#[rstest]
#[serial("cd")]
fn tendril_action_given_path_and_cd_are_tendrils_folder_uses_given_path(
    #[values(ActionMode::Pull, ActionMode::Push)]
    mode: ActionMode,

    #[values(true, false)]
    dry_run: bool,
) {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();

    let current_dir = temp_parent_folder.path().join("CurrentDir");
    let given_folder = temp_parent_folder.path().join("GivenDir");
    std::fs::create_dir_all(&current_dir).unwrap();
    std::fs::create_dir_all(&given_folder).unwrap();
    std::fs::write(current_dir.join("tendrils.json"), "[]").unwrap();
    std::fs::write(given_folder.join("tendrils.json"), "").unwrap();
    std::env::set_current_dir(&current_dir).unwrap();
    assert!(parse_tendrils("[]").unwrap().is_empty());
    assert!(parse_tendrils("").is_err());

    let mut writer = MockWriter::new();
    let path = Some(given_folder.to_str().unwrap().to_string());
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {path, dry_run},
        ActionMode::Push => TendrilsSubcommands::Push {path, dry_run},
        _ => unimplemented!(),
    };
    let args = TendrilCliArgs{
        tendrils_command,
    };

    let expected = "Error: Could not parse the tendrils.json file\n";

    run(args, &mut writer);

    // To free the TempDir from use
    std::env::set_current_dir(temp_parent_folder.path().parent().unwrap()).unwrap();

    assert!(is_tendrils_folder(&current_dir));
    assert!(is_tendrils_folder(&given_folder));
    assert_eq!(writer.all_output, expected);
}

#[rstest]
#[case(ActionMode::Pull)]
#[case(ActionMode::Push)]
fn tendril_action_dry_run_does_not_modify(#[case] mode: ActionMode) {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();

    let mut tendril = Tendril::new("SomeApp", "misc.txt");
    set_all_platform_paths(&mut tendril, &[temp_parent_folder.path().to_path_buf()]);

    let tendrils_json = serde_json::to_string(&[tendril]).unwrap();
    let tendrils_folder = temp_parent_folder.path().join("TendrilsFolder");
    let source = temp_parent_folder.path().join("misc.txt");
    std::fs::create_dir_all(&tendrils_folder).unwrap();
    std::fs::write(tendrils_folder.join("tendrils.json"), tendrils_json).unwrap();
    std::fs::write(&source, "Source file contents").unwrap();

    let mut writer = MockWriter::new();
    let path = Some(tendrils_folder.to_str().unwrap().to_string());
    let dry_run = true;
    let tendrils_command = match mode {
        ActionMode::Pull => TendrilsSubcommands::Pull {path, dry_run},
        ActionMode::Push => TendrilsSubcommands::Push {path, dry_run},
        _ => unimplemented!(),
    };
    let args = TendrilCliArgs{
        tendrils_command,
    };

    run(args, &mut writer);
    assert!(source.exists());
    assert_eq!(writer.all_output, "No local overrides were found.\n");
    assert_eq!(tendrils_folder.read_dir().unwrap().into_iter().count(), 1);
}

// TODO: Test uses_correct_platform_paths (see old commits in pull_tendril_tests)
// TODO: Test multiple_paths_only_copies_first for pull (see old commits in pull_tendril_tests)
// TODO: Test multiple_paths_first_is_missing_returns_not_found_error (see old commits in pull_tendril_tests)
// TODO: Test duplicate_tendrils_returns_duplicate_error_for_second_occurence_onward (see old pull_tests)
// TODO: Test that empty path list returns skipped error (for any action)
