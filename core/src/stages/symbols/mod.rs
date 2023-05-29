use super::{tooling::StageResult, Stage};

/// Symbols stage, responsible for symbols such as `—` and `→`.
#[derive(Debug, Clone, Copy)]
#[allow(clippy::module_name_repetitions)]
pub struct SymbolsStage;

impl Stage for SymbolsStage {
    fn substitute(&self, input: &str) -> StageResult {
        Ok(String::from(input).into())
    }
}
