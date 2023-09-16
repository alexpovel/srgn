use crate::scoped::Scoped;

use super::Stage;

/// Deletes all matches of a given regex.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct DeletionStage {}

impl Scoped for DeletionStage {}

impl Stage for DeletionStage {
    fn substitute(&self, _input: &str) -> String {
        String::new()
    }
}
