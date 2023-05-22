use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Casing {
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Umlaut {
    Ae(Casing),
    Oe(Casing),
    Ue(Casing),
}

impl Display for Umlaut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Umlaut::Ae(Casing::Lower) => 'ä',
                Umlaut::Ae(Casing::Upper) => 'Ä',
                Umlaut::Oe(Casing::Lower) => 'ö',
                Umlaut::Oe(Casing::Upper) => 'Ö',
                Umlaut::Ue(Casing::Lower) => 'ü',
                Umlaut::Ue(Casing::Upper) => 'Ü',
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpecialCharacter {
    Umlaut(Umlaut),
    Eszett(Casing),
}

impl Display for SpecialCharacter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SpecialCharacter::Umlaut(umlaut) => umlaut.to_string(),
                SpecialCharacter::Eszett(Casing::Lower) => String::from('ß'),
                SpecialCharacter::Eszett(Casing::Upper) => String::from('ẞ'),
            }
        )
    }
}
