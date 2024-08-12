pub(crate) trait Writer {
    fn writeln(&mut self, text: &str);
}

pub(crate) struct StdOutWriter {}

impl Writer for StdOutWriter {
    fn writeln(&mut self, text: &str) {
        println!("{text}");
    }
}
