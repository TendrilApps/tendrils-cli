use crate::cli::{run, TendrilCliArgs, TendrilsSubcommands};
use crate::cli::writer::Writer;
use crate::is_tendrils_folder;
use crate::test_utils::get_disposable_folder;
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
#[serial]
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
#[serial]
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

#[test]
#[cfg(not(windows))]
fn push_or_pull_no_path_given_and_no_cd_should_panic() {
    let delete_me = TempDir::new_in(
        get_disposable_folder(),
        "DeleteMe"
    ).unwrap();
    std::env::set_current_dir(delete_me.path()).unwrap();
    std::fs::remove_dir(delete_me.path()).unwrap();

    let mut writer = MockWriter::new();
    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Pull { path: None }
    };
    let expected = "Error: Could not get the current directory\n";

    run(args, &mut writer);

    assert_eq!(writer.all_output, expected);
}

#[test]
fn push_or_pull_given_path_is_not_tendrils_folder_cd_is_prints_message() {
    // TODO: Setup current directory as tendrils folder
    let mut writer = MockWriter::new();
    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Pull {
            path: Some("SomePathThatDoesn'tExist".to_string()),
        }
    };
    let expected = "Error: The given path is not a Tendrils folder\n";

    run(args, &mut writer);

    assert_eq!(writer.all_output, expected);
}

#[test]
fn push_or_pull_given_path_and_cd_are_tendrils_folder_uses_given_path() {
    let temp_parent_folder = TempDir::new_in(
        get_disposable_folder(),
        "ParentFolder"
    ).unwrap();

    let current_dir = temp_parent_folder.path().join("CurrentDir");
    let given_folder = temp_parent_folder.path().join("GivenDir");
    std::fs::create_dir_all(&current_dir).unwrap();
    std::fs::create_dir_all(&given_folder).unwrap();
    std::fs::write(current_dir.join("tendrils.json"), "").unwrap();
    std::fs::write(given_folder.join("tendrils.json"), "").unwrap();

    let mut writer = MockWriter::new();
    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Pull {
            path: Some(given_folder.to_str().unwrap().to_string()),
        }
    };
    let expected = "Error: Could not parse the tendrils.json file\n";
    run(args, &mut writer);

    // TODO: Verify that the correct one was used
    assert!(is_tendrils_folder(&current_dir));
    assert!(is_tendrils_folder(&given_folder));
    assert_eq!(writer.all_output, expected);
}

// TODO: Test uses_correct_platform_paths (see old commits in pull_tendril_tests)
// TODO: Test multiple_paths_only_copies_first for pull (see old commits in pull_tendril_tests)
// TODO: Test multiple_paths_first_is_missing_returns_not_found_error (see old commits in pull_tendril_tests)
// TODO: Test duplicate_tendrils_returns_duplicate_error_for_second_occurence_onward (see old pull_tests)
// TODO: Test that empty path list returns skipped error (for any action)
