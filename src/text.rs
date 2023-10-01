pub trait Unescaper {
    fn unescape(&self, input: &str) -> String;
}
