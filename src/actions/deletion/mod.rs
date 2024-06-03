use super::Action;
use log::info;

/// Deletes everything in the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Deletion {}

impl Action for Deletion {
    fn act(&self, input: &str) -> String {
        info!("Deleting: '{}'", input.escape_debug());
        String::new()
    }
}
