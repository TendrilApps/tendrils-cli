use crate::{execute, Writer};
use crate::cli::{TendrilCliArgs, TendrilsSubcommands};

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
