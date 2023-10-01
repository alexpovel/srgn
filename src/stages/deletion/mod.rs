use log::info;

use super::Stage;

/// Deletes everything in the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct DeletionStage {}

impl Stage for DeletionStage {
    fn process(&self, input: &str) -> String {
        info!("Deleting: '{}'", input);
        String::new()
    }
}
