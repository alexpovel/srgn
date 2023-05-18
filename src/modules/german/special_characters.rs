#[derive(Debug, Clone, Copy)]
pub enum Umlaut {
    Ue,
    Oe,
    Ae,
}

impl Umlaut {
    fn value(&self) -> String {
        String::from(match self {
            Umlaut::Ue => 'ü',
            Umlaut::Oe => 'ö',
            Umlaut::Ae => 'ä',
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpecialCharacter {
    Umlaut(Umlaut),
    Eszett,
}

impl SpecialCharacter {
    pub fn value(&self) -> String {
        match self {
            SpecialCharacter::Umlaut(umlaut) => umlaut.value(),
            SpecialCharacter::Eszett => String::from('ß'),
        }
    }
}
