pub trait Writer {
    fn write(&mut self, text: &str);
}

pub struct StdOutWriter {}

impl Writer for StdOutWriter{
    fn write(&mut self, text: &str) {
        println!("{}", text);
    }
}
