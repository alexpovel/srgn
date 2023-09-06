mod driver;
mod machine;
mod words;

// Re-export symbols.
#[allow(clippy::module_name_repetitions)]
pub use driver::GermanStage;
use words::{LetterCasing, SpecialCharacter, Umlaut, Word};
