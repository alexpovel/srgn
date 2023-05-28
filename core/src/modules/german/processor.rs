use super::word::Replacement;
use crate::{
    modules::{
        german::{
            machine::{StateMachine, Transition},
            word::Replace,
        },
        ProcessResult, TextProcessor,
    },
    util::{
        iteration::{binary_search_uneven, power_set_without_empty},
        strings::{titlecase, WordCasing},
    },
};
use cached::proc_macro::cached;
use cached::SizedCache;
use log::{debug, trace};

static VALID_GERMAN_WORDS: &str = include_str!(concat!(env!("OUT_DIR"), "/de.txt")); // Generated in `build.rs`.

#[derive(Clone, Copy)]
pub struct German;

impl TextProcessor for German {
    fn process(&self, input: &mut String) -> ProcessResult {
        debug!("Working on input '{}'", input.escape_debug());

        // The state machine, much like a missing trailing newline in a file, will
        // misbehave if the very last transition is not an 'external' one (the last word
        // won't be detected properly).
        const INDICATOR: char = '\0';
        input.push(INDICATOR);

        let mut output = String::with_capacity(input.capacity());

        let mut machine = StateMachine::new();

        for char in input.chars() {
            trace!(
                "Beginning processing of character '{}'",
                char.escape_debug()
            );

            let transition = machine.transition(&char);

            trace!("Transition is '{:?}'", transition);

            match transition {
                Transition::External => {
                    output.push(char);
                    continue;
                }
                Transition::Entered | Transition::Internal => {
                    continue;
                }
                Transition::Exited => {
                    debug!("Exited machine: {:?}", machine);

                    let original = machine.current_word().content().to_owned();
                    let word =
                        find_valid_replacement(&original, machine.current_word().replacements())
                            .unwrap_or(original);

                    debug!("Processed word, appending to output: {:?}", &word);
                    output.push_str(&word);

                    // Add back the non-word character that caused the exit transition in the
                    // first place.
                    output.push(char);
                }
            }
        }

        let c = output.pop();
        debug_assert!(
            c == Some(INDICATOR),
            "Processor removed trailing indicator byte."
        );

        debug!("Final output string is '{}'", output.escape_debug());
        *input = output;

        Ok(())
    }
}

fn find_valid_replacement(word: &str, replacements: &[Replacement]) -> Option<String> {
    let replacement_combinations = power_set_without_empty(replacements.iter().cloned());
    debug!("Starting search for valid replacement for word '{}'", word);
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

        if is_valid(&candidate, &contained_in_global_word_list) {
            debug!("Candidate '{}' is valid, returning early", candidate);
            return Some(candidate);
        } else {
            trace!("Candidate '{}' is invalid, trying next one", candidate);
        }
    }

    debug!("No valid replacement found, returning");
    None
}

fn contained_in_global_word_list(word: &str) -> bool {
    binary_search_uneven(word, VALID_GERMAN_WORDS, '\n')
}

// https://github.com/jaemk/cached/issues/135#issuecomment-1315911572
#[cached(
    type = "SizedCache<String, bool>",
    create = "{ SizedCache::with_size(1024) }",
    convert = r#"{ String::from(word) }"#
)]
fn is_valid(word: &str, predicate: &impl Fn(&str) -> bool) -> bool {
    trace!("Trying candidate '{}'", word);

    let casing = WordCasing::try_from(word);
    trace!("Casing of candidate is '{:?}'", casing);

    match casing {
        Ok(WordCasing::AllLowercase) => {
            // Adjectives, verbs, etc.: always lowercase. Nouns are *never* assumed to
            // occur all lowercase (e.g. "laufen"). In any case, there is no further
            // processing we can/want to do (or is there...
            // https://www.youtube.com/watch?v=HLRdruqQfRk).
            predicate(word)
        }
        Ok(WordCasing::AllUppercase | WordCasing::Mixed) => {
            // Before proceeding, convert `SCREAMING` or `MiXeD` words to something
            // sensible, then see from there (e.g. "ABENTEUER" -> "Abenteuer",
            // "üBeRTrIeBeN" -> "Übertrieben"). See `Titlecase` for what happens next.

            let tc = titlecase(word);
            debug_assert!(
                WordCasing::try_from(tc.as_str()) == Ok(WordCasing::Titlecase),
                "Titlecased word, but isn't categorized correctly."
            );

            is_valid(&tc, predicate)
        }
        Ok(WordCasing::Titlecase) => {
            // Regular nouns are normally titlecase, so see if they're found
            // immediately (e.g. "Haus").
            predicate(word)
                // Adjectives and verbs might be titlecased at the beginning of
                // sentences etc. (e.g. "Gut gemacht!" -> we need "gut").
                || is_valid(&word.to_lowercase(), predicate)
                // None of these worked: we might have a compound word. These are
                // *never* assumed to occur as anything but titlecase (e.g.
                // "Hausüberfall").
                || is_valid_compound_word(word, predicate)
        }
        Err(_) => false, // Ran into some unexpected characters...
    }
}

fn is_valid_compound_word(word: &str, predicate: &impl Fn(&str) -> bool) -> bool {
    trace!("Checking if word is valid compound word: '{}'", word);

    if predicate(word) {
        return true;
    }

    let indices = word.char_indices().skip(1);

    // Greedily fetch the longest possible prefix. Otherwise, we short-circuit and might
    // end up looking for (for example) "He" of "Heizölrechnung" and its suffix
    // "izölrechnung" (not a word), whereas we could have found "Heizöl" and "Rechnung"
    // instead.
    let mut highest_valid_index = None;
    for (i, _) in indices {
        let prefix = &word[..i];

        if predicate(prefix) {
            highest_valid_index = Some(i);
        }
    }

    match highest_valid_index {
        Some(i) => {
            let suffix = &word[i..];

            trace!(
                "Prefix '{}' found in word list, seeing if suffix '{}' is valid.",
                &word[..i],
                suffix
            );

            // Compound words are very likely to be made up of nouns, so check that
            // first.
            is_valid_compound_word(&titlecase(suffix), predicate)
                || is_valid_compound_word(suffix, predicate)
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use instrament::instrament;
    use itertools::Itertools;
    use rstest::rstest;

    #[test]
    fn test_words_are_sorted() {
        let original = VALID_GERMAN_WORDS.lines().collect_vec();

        let mut sorted = VALID_GERMAN_WORDS.lines().collect_vec();
        sorted.sort();

        assert_eq!(original, sorted.as_slice());
    }

    #[test]
    fn test_words_are_unique() {
        let original = VALID_GERMAN_WORDS.lines().collect_vec();

        let mut unique = VALID_GERMAN_WORDS.lines().collect_vec();
        unique.sort();
        unique.dedup();

        assert_eq!(original, unique.as_slice());
    }

    #[test]
    fn test_word_list_is_not_filtered() {
        assert!(
            VALID_GERMAN_WORDS.lines().any(|word| word.is_ascii()),
            concat!(
                "Looks like you're using a filtered word list containing only special characters.",
                " The current implementation relies on the full word list (also containing all non-Umlaut words)"
            )
        );
    }

    #[test]
    fn test_is_valid_on_empty_input() {
        assert!(!is_valid("", &contained_in_global_word_list));
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
                "Süßwasserschwimmbäder",
                "مرحبا",
                "你好",
            )]
            word: String
        ) (|data: &TestIsValid| {
                insta::assert_yaml_snapshot!(data.to_string(), is_valid(&word, &contained_in_global_word_list));
            }
        )
    }

    instrament! {
        #[rstest]
        fn test_is_valid_compound_word(
            #[values(
                "Süßwasserschwimmbäder",
                "Mauerdübel",
                "Mauerdübelkübel",
                "Not a compound word",
                "Mauer好",
                "Mauerdjieojoid",
            )]
            word: String
        ) (|data: &TestIsValidCompoundWord| {
                insta::assert_yaml_snapshot!(data.to_string(), is_valid_compound_word(&word, &|w| is_valid(w, &contained_in_global_word_list)));
            }
        )
    }

    instrament! {
        #[rstest]
        fn test_process(
            #[values(
                "\0Kuebel",
                "\0Duebel\0",
                "🤩Duebel",
                "🤩Duebel🤐",
                "Dübel",
                "Abenteuer sind toll!",
                "Koeffizient",
                "kongruent",
                "Ich mag Aepfel, aber nicht Aerger.",
                "Ich mag AEPFEL!! 😍",
                "Wer mag Aepfel?!",
                "Was sind aepfel?",
                "Oel ist ein wichtiger Bestandteil von Oel.",
                "WARUM SCHLIESSEN WIR NICHT AB?",
                "Wir schliessen nicht ab.",
                "WiR sChLieSsEn ab!",
                "WiR sChLiesSEn vieLleEcHt aB.",
                "Suess!",
            )]
            word: String
        ) (|data: &TestProcess| {
                let mut input = word.clone();
                German{}.process(&mut input).unwrap();
                insta::assert_yaml_snapshot!(data.to_string(), input);
            }
        )
    }
}
