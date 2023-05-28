use itertools::Itertools;
use std::fmt::Display;

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
                Umlaut::Ae(LetterCasing::Lower) => 'ä',
                Umlaut::Ae(LetterCasing::Upper) => 'Ä',
                Umlaut::Oe(LetterCasing::Lower) => 'ö',
                Umlaut::Oe(LetterCasing::Upper) => 'Ö',
                Umlaut::Ue(LetterCasing::Lower) => 'ü',
                Umlaut::Ue(LetterCasing::Upper) => 'Ü',
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
                SpecialCharacter::Umlaut(umlaut) => umlaut.to_string(),
                SpecialCharacter::Eszett(LetterCasing::Lower) => String::from('ß'),
                SpecialCharacter::Eszett(LetterCasing::Upper) => String::from('ẞ'),
            }
        )
    }
}

#[derive(Debug)]
pub(super) struct Word {
    content: String,
    replacements: Vec<Replacement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Replacement {
    span: Span,
    content: SpecialCharacter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Span {
    start: usize,
    end: usize,
}

impl Word {
    /// Clears the word's contents while retaining any allocated capacities.
    pub fn clear(&mut self) {
        self.content.clear();
        self.replacements.clear();
    }

    pub fn push(&mut self, character: char) {
        self.content.push(character);
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn add_replacement(&mut self, start: usize, end: usize, content: SpecialCharacter) {
        self.replacements.push(Replacement {
            span: Span { start, end },
            content,
        });
    }

    pub fn replacements(&self) -> &Vec<Replacement> {
        &self.replacements
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Default for Word {
    fn default() -> Self {
        Self {
            content: String::with_capacity(crate::EXPECTABLE_MAXIMUM_WORD_LENGTH_BYTES as usize),
            replacements: Vec::with_capacity(crate::EXPECTABLE_MAXIMUM_MATCHES_PER_WORD as usize),
        }
    }
}

impl Replacement {
    pub fn start(&self) -> usize {
        self.span.start
    }

    pub fn end(&self) -> usize {
        self.span.end
    }

    pub fn content(&self) -> &SpecialCharacter {
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
            cloned.sort_by_key(|replacement| replacement.start());
            assert_eq!(cloned, replacements);
        }

        // We are replacing starting from behind. Otherwise, earlier indices invalidate
        // later ones.
        for replacement in replacements.into_iter().rev() {
            self.apply_replacement(&replacement);
        }
    }
}
