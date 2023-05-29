use super::{Stage, StageResult};

#[derive(Clone, Copy)]
pub struct Symbols;

impl Stage for Symbols {
    fn substitute(&self, input: &str) -> StageResult {
        Ok(String::from(input).into())
    }
}
