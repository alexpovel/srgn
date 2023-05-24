mod machine;
mod processor;
mod special_characters;
mod word;

// Re-export symbols.
pub use processor::German;
pub(self) use special_characters::{LetterCasing, SpecialCharacter, Umlaut};
pub(self) use word::Word;
