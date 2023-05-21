use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Umlaut {
    Ue,
    Oe,
    Ae,
}

impl Display for Umlaut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Umlaut::Ue => 'ü',
                Umlaut::Oe => 'ö',
                Umlaut::Ae => 'ä',
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum SpecialCharacter {
    Umlaut(Umlaut),
    Eszett,
}

impl Display for SpecialCharacter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SpecialCharacter::Umlaut(umlaut) => umlaut.to_string(),
                SpecialCharacter::Eszett => String::from('ß'),
            }
        )
    }
}
