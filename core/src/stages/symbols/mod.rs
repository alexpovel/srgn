use super::{Stage, StageResult};

#[derive(Clone, Copy)]
pub struct Symbols;

impl Stage for Symbols {
    fn process(&self, _input: &mut String) -> StageResult {
        Ok(())
    }
}
