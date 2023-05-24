use super::{ProcessResult, TextProcessor};

#[derive(Clone, Copy)]
pub struct Symbols;

impl TextProcessor for Symbols {
    fn process(&self, _input: &mut String) -> ProcessResult {
        Ok(())
    }
}
