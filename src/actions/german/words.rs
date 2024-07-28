use itertools::Itertools;
use std::{fmt::Display, ops::Range};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum WordCasing {
    AllLowercase,
    AllUppercase,
    Titlecase,
    Mixed,
}

/// Error conditions when parsing a string into a `WordCasing`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum WordCasingError {
    /// The string is empty.
    EmptyString,
    /// The string contains characters with undecidable casing.
    ///
    /// These are all sorts of characters, even ASCII ones: `!`, `?`, emojis, ...
    UndecidableCasing,
}

impl TryFrom<&str> for WordCasing {
    type Error = WordCasingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(WordCasingError::EmptyString);
        }

        let mut has_lowercase = false;
        let mut has_uppercase = false;
        let mut is_titlecase = true;

        for (i, c) in value.chars().enumerate() {
            if c.is_lowercase() {
                has_lowercase = true;

                if i == 0 {
                    is_titlecase = false;
                }
            } else if c.is_uppercase() {
                has_uppercase = true;

                if i != 0 {
                    is_titlecase = false;
                }
            } else {
                return Err(WordCasingError::UndecidableCasing);
            }
        }

        match (is_titlecase, has_lowercase, has_uppercase) {
            (true, _, _) => Ok(Self::Titlecase),
            (_, true, false) => Ok(Self::AllLowercase),
            (_, false, true) => Ok(Self::AllUppercase),
            (_, true, true) => Ok(Self::Mixed),
            (_, false, false) => unreachable!("Impossible case: any non-empty string has either lower- or uppercase or returned an `Err` early."),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LetterCasing {
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Umlaut {
    Ae(LetterCasing),
    Oe(LetterCasing),
    Ue(LetterCasing),
}

impl Display for Umlaut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Ae(LetterCasing::Lower) => 'ä',
                Self::Ae(LetterCasing::Upper) => 'Ä',
                Self::Oe(LetterCasing::Lower) => 'ö',
                Self::Oe(LetterCasing::Upper) => 'Ö',
                Self::Ue(LetterCasing::Lower) => 'ü',
                Self::Ue(LetterCasing::Upper) => 'Ü',
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpecialCharacter {
    Umlaut(Umlaut),
    Eszett(LetterCasing),
}

impl Display for SpecialCharacter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Umlaut(umlaut) => umlaut.to_string(),
                Self::Eszett(LetterCasing::Lower) => String::from('ß'),
                Self::Eszett(LetterCasing::Upper) => String::from('ẞ'),
            }
        )
    }
}

#[derive(Debug)]
pub(super) struct Word {
    content: String,
    replacements: Vec<Replacement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Replacement {
    span: Range<usize>,
    content: SpecialCharacter,
}

impl Word {
    /// Clears the word's contents while retaining any allocated capacities.
    pub(super) fn clear(&mut self) {
        self.content.clear();
        self.replacements.clear();
    }

    pub(super) fn push(&mut self, character: char) {
        self.content.push(character);
    }

    pub(super) fn len(&self) -> usize {
        self.content.len()
    }

    pub(super) fn add_replacement(&mut self, start: usize, end: usize, content: SpecialCharacter) {
        self.replacements.push(Replacement {
            span: Range { start, end },
            content,
        });
    }

    pub(super) const fn replacements(&self) -> &Vec<Replacement> {
        &self.replacements
    }

    pub(super) fn content(&self) -> &str {
        &self.content
    }
}

impl Default for Word {
    fn default() -> Self {
        Self {
            content: String::with_capacity(super::EXPECTABLE_AVERAGE_WORD_LENGTH_BYTES as usize),
            replacements: Vec::with_capacity(super::EXPECTABLE_AVERAGE_MATCHES_PER_WORD as usize),
        }
    }
}

impl Replacement {
    pub(super) const fn start(&self) -> usize {
        self.span.start
    }

    pub(super) const fn end(&self) -> usize {
        self.span.end
    }

    pub(super) const fn content(&self) -> &SpecialCharacter {
        &self.content
    }
}

pub(super) trait Replace {
    fn apply_replacement(&mut self, replacement: &Replacement);
    fn apply_replacements<T>(&mut self, replacements: T)
    where
        T: IntoIterator<Item = Replacement>,
        T::IntoIter: DoubleEndedIterator<Item = Replacement>;
}

impl Replace for String {
    fn apply_replacement(&mut self, replacement: &Replacement) {
        self.replace_range(
            replacement.start()..replacement.end(),
            &replacement.content().to_string(),
        );
    }

    fn apply_replacements<I>(&mut self, replacements: I)
    where
        I: IntoIterator<Item = Replacement>,
        I::IntoIter: DoubleEndedIterator<Item = Replacement>,
    {
        let replacements = replacements.into_iter().collect_vec();

        // Assert sorting, such that reversing actually does the right thing.
        if cfg!(debug_assertions) {
            let mut cloned = replacements.iter().cloned().collect_vec();
            cloned.sort_by_key(Replacement::start);
            assert_eq!(cloned, replacements);
        }

        // We are replacing starting from behind. Otherwise, earlier indices invalidate
        // later ones.
        for replacement in replacements.into_iter().rev() {
            self.apply_replacement(&replacement);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WordCasing::*;
    use super::WordCasingError::*;
    use super::*;
    use rstest::rstest;

    #[rstest]
    // Lowercase
    #[case("hello", Ok(AllLowercase))]
    #[case("uebel", Ok(AllLowercase))]
    #[case("übel", Ok(AllLowercase))]
    #[case("ßuper", Ok(AllLowercase))]
    //
    // Uppercase
    #[case("SCREAMING", Ok(AllUppercase))]
    //
    // Mixed
    #[case("bItTe", Ok(Mixed))]
    #[case("dANKE", Ok(Mixed))]
    //
    // Titlecase
    #[case("ẞuperduper", Ok(Titlecase))]
    #[case("ẞß", Ok(Titlecase))] // Eszett works
    //
    // Error conditions
    #[case("WOW!!", Err(UndecidableCasing))]
    #[case("😀", Err(UndecidableCasing))]
    #[case("", Err(EmptyString))]
    fn test_word_casing_from_string(
        #[case] input: &str,
        #[case] expected: Result<WordCasing, WordCasingError>,
    ) {
        assert_eq!(WordCasing::try_from(input), expected);
    }
}
#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)] // in `proptest` macro, cannot be avoided
mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]
        #[test]
        fn test_wordcasing_does_not_panic(
            // https://docs.rs/regex/latest/regex/#matching-one-character
            // https://www.unicode.org/reports/tr44/tr44-24.html#General_Category_Values
            input in r"\p{Any}*"
        ) {
            let _ = WordCasing::try_from(input.as_str());
        }
    }
}
