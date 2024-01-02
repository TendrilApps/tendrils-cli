pub trait Writer {
    fn write(&self, text: &str);
}

pub struct StdOutWriter {}

impl Writer for StdOutWriter{
    fn write(&self, text: &str) {
        println!("{}", text);
    }
}
