use super::Action;

/// Returns its input unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Print {}

impl Action for Print {
    fn act(&self, input: &str) -> String {
        // We wouldn't need to copy here, but that's what the trait requires.
        input.to_owned()
    }
}
