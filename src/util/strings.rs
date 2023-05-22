pub fn first_char(word: &str) -> char {
    word.chars()
        .next()
        .expect("Cannot get first char of empty word")
}

#[derive(Debug, Clone, Copy)]
enum Casing {
    Lower,
    Upper,
}

fn first_char_with_case(word: &str, case: Casing) -> String {
    let mut new = String::with_capacity(word.len());
    let c = first_char(word);

    match case {
        Casing::Lower => {
            let c = c.to_string().to_lowercase();
            new.push_str(&c);
        }
        Casing::Upper => {
            let c = c.to_string().to_uppercase();
            new.push_str(&c);
        }
    };

    new.push_str(&word[c.len_utf8()..]);

    new
}

pub fn lowercase_first_char(word: &str) -> String {
    first_char_with_case(word, Casing::Lower)
}

pub fn uppercase_first_char(word: &str) -> String {
    first_char_with_case(word, Casing::Upper)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde::Serialize;

    use super::*;

    use crate::testing::instrament;

    impl Serialize for Casing {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Casing::Lower => serializer.serialize_str("lower"),
                Casing::Upper => serializer.serialize_str("upper"),
            }
        }
    }

    instrament! {
            #[rstest]
            fn test_first_char(
            #[values(
                "Hello",
                "Übel",
                "Uebel",
                "😀",
                "ßuper",
                "ẞuperduper",
            )]
                word: String
            ) (|data: &TestFirstChar| {
                insta::assert_snapshot!(data.to_string(), first_char(&word).to_string());
            }
        )
    }

    #[test]
    #[should_panic]
    fn test_first_char_panics_on_empty_string() {
        first_char("");
    }

    instrament! {
            #[rstest]
            fn test_lowercasing(
            #[values(
                "Hello",
                "Übel",
                "Uebel",
                "😀",
                "ßuper",
                "ẞuperduper",
            )]
                word: String
            ) (|data: &TestLowercasing| {
                insta::assert_snapshot!(data.to_string(), lowercase_first_char(&word));
            }
        )
    }

    instrament! {
        #[rstest]
        fn test_uppercasing(
        #[values(
            "hello",
            "übel",
            "uebel",
            "😀",
            "ßuper",
            "ẞuperduper",
        )]
            word: String
        ) (|data: &TestUppercasing| {
                insta::assert_snapshot!(data.to_string(), uppercase_first_char(&word));
            }
        )
    }
}
