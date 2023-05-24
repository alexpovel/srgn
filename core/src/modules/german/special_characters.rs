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
