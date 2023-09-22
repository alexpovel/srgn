mod driver;
mod machine;
mod words;

const EXPECTABLE_AVERAGE_WORD_LENGTH_BYTES: u8 = 16;
const EXPECTABLE_AVERAGE_MATCHES_PER_WORD: u8 = 2;

// Re-export symbols.
#[allow(clippy::module_name_repetitions)]
pub use driver::GermanStage;
use words::{LetterCasing, SpecialCharacter, Umlaut, Word};
