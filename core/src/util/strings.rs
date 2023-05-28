#[derive(Debug, PartialEq, Eq)]
pub(crate) enum WordCasing {
    AllLowercase,
    AllUppercase,
    Titlecase,
    Mixed,
}

impl TryFrom<&str> for WordCasing {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err("String is empty");
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
                return Err("String contains characters with undecidable casing");
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

#[cfg(test)]
mod tests {
    use super::*;
    use common::instrament;
    use rstest::rstest;
    use serde::Serialize;

    impl Serialize for WordCasing {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Self::AllLowercase => serializer.serialize_str("AllLowercase"),
                Self::AllUppercase => serializer.serialize_str("AllUppercase"),
                Self::Titlecase => serializer.serialize_str("Titlecase"),
                Self::Mixed => serializer.serialize_str("Mixed"),
            }
        }
    }

    instrament! {
            #[rstest]
            fn test_word_casing_from_string(
            #[values(
                "hello",
                "bItTe",
                "dANKE",
                "übel",
                "uebel",
                "😀",
                "ßuper",
                "ẞuperduper",
                "WOW!!",
                "SCREAMING",
                "ẞß",
                "",
            )]
                word: String
            ) (|data: &TestWordCasingFromString| {
                insta::assert_yaml_snapshot!(data.to_string(), WordCasing::try_from(word.as_str()));
            }
        )
    }
}
