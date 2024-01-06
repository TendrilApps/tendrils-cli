use crate::{run, get_disposable_folder, Writer};
use crate::cli::{TendrilCliArgs, TendrilsSubcommands};
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

    run(args, &mut writer);
}
