use crate::{execute, get_disposable_folder, Writer};
use crate::cli::{TendrilCliArgs, TendrilsSubcommands};
use serial_test::serial;
use tempdir::TempDir;

const TENDRILS_VAR_NAME: &str = "TENDRILS_FOLDER";

struct MockWriter {
    output: String,
}

impl MockWriter {
    fn new() -> MockWriter {
        MockWriter {
            output: "".to_string(),
        }
    }
}

impl Writer for MockWriter {
    fn write(&mut self, text: &str) {
        self.output = text.to_string();
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
        "The '{}' environment variable is not set.", TENDRILS_VAR_NAME
    );

    execute(args, &mut writer);

    assert_eq!(writer.output, expected);
}

#[test]
#[serial]
fn path_with_env_var_set_prints_path() {
    let mut writer = MockWriter::new();
    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Path
    };
    std::env::set_var(TENDRILS_VAR_NAME, "SomePath");
    let expected = "SomePath";

    execute(args, &mut writer);

    assert_eq!(writer.output, expected);
}

#[test]
#[should_panic(expected = "Error: Could not get the current directory")]
fn push_or_pull_no_path_given_and_no_cd_should_panic() {
    let temp = TempDir::new_in(
        get_disposable_folder(),
        "Temp"
    ).unwrap();
    let delete_dir = temp.path().join("DeleteMe");
    std::fs::create_dir_all(&delete_dir).unwrap();
    std::env::set_current_dir(&delete_dir).unwrap();
    std::fs::remove_dir(&delete_dir).unwrap();

    let mut writer = MockWriter::new();
    let args = TendrilCliArgs{
        tendrils_command: TendrilsSubcommands::Pull { path: None }
    };

    execute(args, &mut writer);
}
