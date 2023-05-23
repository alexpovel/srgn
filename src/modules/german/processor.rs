use log::{debug, trace};

use crate::{
    modules::{
        german::{
            machine::{StateMachine, Transition},
            word::Replace,
        },
        ProcessResult, TextProcessor,
    },
    util::{
        iteration::power_set,
        strings::{first_char, lowercase_first_char, uppercase_first_char},
    },
};

use super::word::Replacement;

const VALID_GERMAN_WORDS: &[&str] = include!(concat!(env!("OUT_DIR"), "/de.in")); // Generated in `build.rs`.

pub struct German;

impl TextProcessor for German {
    fn process(&self, input: &mut String) -> ProcessResult {
        debug!("Working on input '{}'", input);

        // The state machine, much like a missing trailing newline in a file, will
        // misbehave if the very last transition is not an 'external' one (the last word
        // won't be detected properly).
        const INDICATOR: char = '\0';
        input.push(INDICATOR);

        let mut output = String::with_capacity(input.capacity());

        let mut machine = StateMachine::new();

        for char in input.chars() {
            trace!("Beginning processing of character '{}'", char);

            let transition = machine.transition(&char);

            trace!("Transition is '{:?}'", transition);

            match transition {
                Some(Transition::External) => {
                    output.push(char);
                    continue;
                }
                Some(Transition::Entered | Transition::Internal) => {
                    continue;
                }
                Some(Transition::Exited) => {
                    trace!("Exited word: {:?}", machine.current_word());

                    let original = machine.current_word().content().to_owned();
                    let word = find_valid_replacement(
                        &original,
                        machine.current_word().replacements(),
                        VALID_GERMAN_WORDS,
                    )
                    .unwrap_or(original);

                    output.push_str(&word);

                    // Add back the non-word character that caused the exit transition in the
                    // first place.
                    output.push(char);
                }
                None => unreachable!("After initial transition, must have `Some`."),
            }
        }

        debug!("Final string is '{}'", output);
        *input = output;

        let c = input.pop();
        debug_assert!(
            c == Some(INDICATOR),
            "Processor removed trailing indicator byte."
        );

        Ok(())
    }
}

fn find_valid_replacement(
    word: &str,
    replacements: &[Replacement],
    valid_words: &[&str],
) -> Option<String> {
    let replacement_combinations = power_set(
        replacements.iter().cloned(),
        // Exclude empty set, unnecessary work:
        false,
    );
    trace!(
        "All replacement combinations to try: {:?}",
        replacement_combinations
    );

    for replacements in replacement_combinations {
        let mut candidate = word.to_owned();
        candidate.apply_replacements(replacements);
        trace!(
            "Replaced candidate word, now is: '{}'. Starting validity check.",
            candidate
        );

        if is_valid(&candidate, valid_words) {
            trace!("Candidate is valid, returning.");
            return Some(candidate);
        } else {
            trace!("Candidate is invalid, trying next one.");
        }
    }

    trace!("No valid replacement found, returning.");
    None
}

fn is_valid(word: &str, valid_words: &[&str]) -> bool {
    debug_assert!(
        valid_words.iter().any(|word| word.is_ascii()),
        "Looks like you're using a filtered word list. This function only works with the full word list (also containing all non-Umlaut words)"
    );

    trace!("Trying candidate '{}'...", word);

    // Pretty much all ordinarily lowercase words *might* appear uppercased, e.g. at the
    // beginning of sentences. For example: "Uebel!" -> "Übel!", even though only "übel"
    // is in the dictionary.
    if first_char(word).is_uppercase() && is_valid(&lowercase_first_char(word), valid_words) {
        trace!("Candidate '{}' is valid when lowercased.", word);
        return true;
    }

    let search = |word| valid_words.binary_search(&word).is_ok();

    if search(word) {
        trace!("Found candidate '{}' in word list, is valid.", word);
        return true;
    }

    for (i, _) in word
        .char_indices()
        // Skip, as `prefix` empty on first iteration otherwise, which is wasted work.
        .skip(1)
    {
        let prefix = &word[..i];
        trace!("Trying prefix '{}'", prefix);

        if search(prefix) {
            let suffix = &word[i..];

            trace!(
                "Prefix found in word list, seeing if (uppercased) suffix '{}' is valid.",
                suffix
            );

            // We uppercase to detect e.g. `Mauerdübel`, where after the first iteration
            // we'd have `Mauer` and `dübel`, with only `Dübel` being valid.
            //
            // Next recursion will test both lower- and this uppercased version, so also
            // words like `Mauergrün` are valid, where `grün` is in the dictionary but
            // `Grün` *might* not be, for example.
            return is_valid(&uppercase_first_char(suffix), valid_words);
        }

        trace!("Prefix not found in word list, trying next.");
    }

    false
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::testing::instrament;

    use super::*;

    #[test]
    fn test_words_are_sorted() {
        let mut sorted = VALID_GERMAN_WORDS.to_vec();
        sorted.sort();
        assert_eq!(VALID_GERMAN_WORDS, sorted.as_slice());
    }

    #[test]
    fn test_words_are_unique() {
        let mut unique = VALID_GERMAN_WORDS.to_vec();
        unique.sort();
        unique.dedup();
        assert_eq!(VALID_GERMAN_WORDS, unique.as_slice());
    }

    #[test]
    #[should_panic]
    fn test_is_valid_panics_on_filtered_word_list() {
        let words = &["Önly", "speciäl", "wörds"];
        is_valid("Doesn't matter, this will panic.", words);
    }

    #[test]
    #[should_panic]
    fn test_is_valid_panics_on_empty_input() {
        is_valid("", VALID_GERMAN_WORDS);
    }

    instrament! {
        #[rstest]
        fn test_is_valid(
            #[values(
                "????",
                "\0",
                "\0Dübel",
                "\0Dübel\0",
                "🤩Dübel",
                "🤩Dübel🤐",
                "😎",
                "dröge",
                "DüBeL",
                "Dübel\0",
                "Duebel",
                "kindergarten",
                "Koeffizient",
                "kongruent",
                "Kübel",
                "Mauer",
                "Mauer😂",
                "Mauerdübel",
                "Mauerdübelkübel",
                "Maür",
                "Maürdübelkübel",
                "messgerät",
                "No\nway",
                "مرحبا",
                "你好",
            )]
            word: String
        ) (|data: &TestIsValid| {
                insta::assert_yaml_snapshot!(data.to_string(), is_valid(&word, VALID_GERMAN_WORDS));
            }
        )
    }
}
