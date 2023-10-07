use log::info;

use super::Action;

/// Deletes everything in the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Deletion {}

impl Action for Deletion {
    fn act(&self, input: &str) -> String {
        info!("Deleting: '{}'", input);
        String::new()
    }
}
