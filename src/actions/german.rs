mod driver;
mod machine;
mod words;

const EXPECTABLE_AVERAGE_WORD_LENGTH_BYTES: u8 = 16;
const EXPECTABLE_AVERAGE_MATCHES_PER_WORD: u8 = 2;

// Re-export symbols.
pub use driver::German;
use words::{LetterCasing, SpecialCharacter, Umlaut, Word};
