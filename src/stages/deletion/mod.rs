use super::{tooling::StageResult, Stage};
use regex::Regex;

/// Deletes all matches of a given regex.
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct DeletionStage {
    pattern: Regex,
}

impl Stage for DeletionStage {
    fn substitute(&self, input: &str) -> StageResult {
        Ok(self.pattern.replace_all(input, "").to_string().into())
    }
}

impl DeletionStage {
    /// Create a new [`DeletionStage`].
    ///
    /// # Arguments
    ///
    /// * `pattern`: The regex to use for deletion.
    #[must_use]
    pub fn new(pattern: Regex) -> Self {
        Self { pattern }
    }
}
