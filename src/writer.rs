pub(crate) trait Writer: Clone {
    /// Write to stdout with newline.
    fn writeln(&mut self, text: &str);

    /// Write to stderr.
    fn ewrite(&mut self, text: &str);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StdOutWriter {}

impl Writer for StdOutWriter {
    fn ewrite(&mut self, text: &str) {
        eprint!("{text}");
    }

    fn writeln(&mut self, text: &str) {
        println!("{text}");
    }
}
