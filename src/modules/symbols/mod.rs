use super::{ProcessResult, TextProcessor};

pub struct Symbols;

impl TextProcessor for Symbols {
    fn process(&self, _input: &mut String) -> ProcessResult {
        Ok(())
    }
}
