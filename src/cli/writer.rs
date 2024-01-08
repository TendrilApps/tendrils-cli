pub trait Writer {
    fn writeln(&mut self, text: &str);
}

pub struct StdOutWriter {}

impl Writer for StdOutWriter{
    fn writeln(&mut self, text: &str) {
        println!("{}", text);
    }
}
