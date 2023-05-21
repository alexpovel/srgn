use log::{debug, trace};

use crate::{
    iteration::power_set,
    modules::{german::word::Replace, TextProcessor},
};

use super::{SpecialCharacter, Umlaut, Word};

#[derive(Default, Debug)]
enum State {
    #[default]
    Other,
    Word(Option<Potential>),
}

#[derive(Debug)]
struct Potential(SpecialCharacter);

#[derive(Debug)]
enum Transition {
    // Entered a word.
    Entered,
    // Exited a word.
    Exited,
    // Within two word characters.
    Internal,
    // Between two non-word characters.
    External,
}

impl Transition {
    fn from_states(from: &State, to: &State) -> Self {
        match (from, to) {
            (State::Word(_), State::Other) => Transition::Exited,
            (State::Other, State::Word(_)) => Transition::Entered,
            (State::Word(_), State::Word(_)) => Transition::Internal,
            (State::Other, State::Other) => Transition::External,
        }
    }
}

type MachineInput = char;

struct StateMachine {
    state: State,
    word: Word,
    transition: Option<Transition>,
}

impl StateMachine {
    fn new() -> Self {
        Self {
            state: State::default(),
            word: Word::default(),
            transition: None,
        }
    }

    fn pre_transition(&mut self) {
        if let State::Other = self.state {
            self.word.clear();
        };
    }

    fn transition(&mut self, input: &MachineInput) -> &Option<Transition> {
        self.pre_transition();

        let next = match (&self.state, input) {
            (
                State::Word(None)
                | State::Word(Some(Potential(SpecialCharacter::Umlaut(_))))
                | State::Other,
                c @ 'o' | c @ 'u' | c @ 'a' | c @ 's',
            ) => State::Word(Some(Potential(match c {
                'o' => SpecialCharacter::Umlaut(Umlaut::Oe),
                'u' => SpecialCharacter::Umlaut(Umlaut::Ue),
                'a' => SpecialCharacter::Umlaut(Umlaut::Ae),
                's' => SpecialCharacter::Eszett,
                _ => unreachable!("Protected by outer match statement."),
            }))),
            //
            (State::Word(Some(Potential(SpecialCharacter::Eszett))), c @ 's') => {
                let pos = self.word.len();

                let start = pos - c.len_utf8(); // Previous char same as current `c`
                let end = pos + c.len_utf8();
                self.word
                    .add_replacement(start, end, SpecialCharacter::Eszett);
                State::Word(None)
            }
            (State::Word(Some(Potential(SpecialCharacter::Umlaut(umlaut)))), c @ 'e') => {
                let pos = self.word.len();

                const LENGTH_OF_PREVIOUS_CHARACTER: usize = 1;
                debug_assert!(
                    'o'.len_utf8() == LENGTH_OF_PREVIOUS_CHARACTER
                        && 'u'.len_utf8() == LENGTH_OF_PREVIOUS_CHARACTER
                        && 'a'.len_utf8() == LENGTH_OF_PREVIOUS_CHARACTER
                );

                let start = pos - LENGTH_OF_PREVIOUS_CHARACTER;
                let end = pos + c.len_utf8();
                self.word
                    .add_replacement(start, end, SpecialCharacter::Umlaut(*umlaut));
                State::Word(None)
            }
            //
            (_, c) if c.is_alphabetic() => State::Word(None),
            (_, _) => State::Other,
        };

        self.post_transition(input, &next);

        self.state = next;
        &self.transition
    }

    fn post_transition(&mut self, input: &MachineInput, next: &State) {
        self.transition = Some(Transition::from_states(&self.state, next));

        if let Some(Transition::Entered | Transition::Internal) = self.transition {
            self.word.push(*input);
        };
    }
}

pub struct German;

// Generated in `build.rs`.
const WORDS: &[&str] = include!(concat!(env!("OUT_DIR"), "/de.in"));

fn is_valid(word: &str, words: &[&str]) -> bool {
    debug_assert!(
        words.iter().any(|word| word.is_ascii()),
        "Looks like you're using a filtered word list. This function only works with the full word list (also containing all non-Umlaut words)"
    );

    trace!("Trying candidate '{}'...", word);

    if words.binary_search(&word).is_ok() {
        trace!("Found candidate '{}' in word list, is valid.", word);
        return true;
    }

    // Skip initial, else initial `prefix` slice is empty.
    for (i, _) in word.char_indices().skip(1) {
        let prefix = &word[..i];
        trace!("Trying prefix '{}'", prefix);

        if words.binary_search(&prefix).is_ok() {
            let suffix = &word[i..];

            // We cannot get around copying the whole string (`String.remove`), as the
            // new, uppercased character might have a different byte length. It might
            // therefore not fit into the newly open slot at index 0.
            let mut uc_suffix = suffix.to_string();
            uc_suffix = uc_suffix.remove(0).to_uppercase().to_string() + &uc_suffix;

            trace!(
                "Prefix found in word list, seeing if either original '{}' or uppercased suffix '{}' is valid.",
                suffix,
                uc_suffix
            );

            // Recursively forks the search into two branches. The uppercase version is
            // likelier to be a hit, hence try first in hopes of a short circuit.
            return is_valid(&uc_suffix, words) || is_valid(suffix, words);
        }

        trace!("Prefix not found in word list, trying next.");
    }

    false
}

impl TextProcessor for German {
    fn process(&self, input: &mut String) -> bool {
        debug!("Working on input '{}'", input);

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
                Some(Transition::Exited) => (),
                None => unreachable!("After initial transition, must have `Some`."),
            }

            trace!("Exited word: {:?}", machine.word);

            let replacement_combinations = power_set(
                machine.word.replacements().clone().into_iter(),
                // Exclude empty set, unnecessary work:
                false,
            );
            trace!(
                "All replacement combinations to try: {:?}",
                replacement_combinations
            );

            let get_fresh_candidate = || machine.word.content().clone();
            let mut candidate = get_fresh_candidate();

            for replacement_combination in replacement_combinations {
                candidate.apply_replacements(replacement_combination);
                trace!(
                    "Replaced candidate word, now is: '{}'. Starting validity check.",
                    candidate
                );

                if is_valid(&candidate, WORDS) {
                    trace!("Candidate is valid word, exiting search.");
                    break;
                }

                candidate = get_fresh_candidate();
            }

            output.push_str(&candidate);

            // Add back the non-word character that caused the exit transition.
            output.push(char);
        }

        debug!("Final string is '{}'", output);
        *input = output;
        true
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::testing::instrament;

    use super::*;

    #[test]
    fn test_words_are_sorted() {
        let mut sorted = WORDS.to_vec();
        sorted.sort();
        assert_eq!(WORDS, sorted.as_slice());
    }

    #[test]
    fn test_words_are_unique() {
        let mut unique = WORDS.to_vec();
        unique.sort();
        unique.dedup();
        assert_eq!(WORDS, unique.as_slice());
    }

    #[test]
    #[should_panic]
    fn test_is_valid_panics_on_filtered_word_list() {
        let words = &["Önly", "speciäl", "wörds"];
        is_valid("Doesn't matter, this will panic.", words);
    }

    instrament! {
        #[rstest]
        fn test_is_valid(
            #[values(
                "????",
                "",
                "\0",
                "\0Dübel",
                "\0Dübel\0",
                "🤩Dübel",
                "🤩Dübel🤐",
                "😎",
                "dübel",
                "Dübel",
                "DüBeL",
                "Dübel\0",
                "Duebel",
                "kindergarten",
                "Kindergarten",
                "Koeffizient",
                "kongruent",
                "Mauer",
                "Mauer😂",
                "Mauerdübel",
                "Mauerdübelkübel",
                "Maür",
                "Maürdübelkübel",
                "No\nway",
                "مرحبا",
                "你好",
            )]
            word: String
        ) (|data: &TestIsValid| {
                insta::assert_yaml_snapshot!(data.to_string(), is_valid(&word, WORDS));
            }
        )
    }
}
