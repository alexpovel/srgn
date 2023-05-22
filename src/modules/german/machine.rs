use log::{debug, trace};

use crate::{
    modules::{german::word::Replace, TextProcessor},
    util::{
        iteration::power_set,
        strings::{first_char, lowercase_first_char, uppercase_first_char},
    },
};

use super::{
    Casing::Lower, Casing::Upper, SpecialCharacter, SpecialCharacter::Eszett,
    SpecialCharacter::Umlaut, Umlaut::Ae, Umlaut::Oe, Umlaut::Ue, Word,
};

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
            (State::Word(Some(Potential(Umlaut(umlaut)))), c @ 'e' | c @ 'E') => {
                let pos = self.word.len();

                const LENGTH_OF_PREVIOUS_CHARACTER: usize = 1;
                debug_assert!(
                    'o'.len_utf8() == LENGTH_OF_PREVIOUS_CHARACTER
                        && 'u'.len_utf8() == LENGTH_OF_PREVIOUS_CHARACTER
                        && 'a'.len_utf8() == LENGTH_OF_PREVIOUS_CHARACTER
                );

                let start = pos - LENGTH_OF_PREVIOUS_CHARACTER;
                let end = pos + c.len_utf8();
                self.word.add_replacement(start, end, Umlaut(*umlaut));
                State::Word(None)
            }
            (State::Word(None) | State::Word(Some(Potential(Umlaut(_)))) | State::Other, c) => {
                match c {
                    'a' => State::Word(Some(Potential(Umlaut(Ae(Lower))))),
                    'A' => State::Word(Some(Potential(Umlaut(Ae(Upper))))),
                    'o' => State::Word(Some(Potential(Umlaut(Oe(Lower))))),
                    'O' => State::Word(Some(Potential(Umlaut(Oe(Upper))))),
                    'u' => State::Word(Some(Potential(Umlaut(Ue(Lower))))),
                    'U' => State::Word(Some(Potential(Umlaut(Ue(Upper))))),
                    's' => State::Word(Some(Potential(Eszett(Lower)))),
                    'S' => State::Word(Some(Potential(Eszett(Upper)))),
                    c if c.is_alphabetic() => State::Word(None),
                    _ => State::Other,
                }
            }
            (State::Word(Some(Potential(Eszett(casing)))), c @ 's' | c @ 'S') => {
                let pos = self.word.len();

                let start = pos - c.len_utf8(); // Previous char same as current `c`
                let end = pos + c.len_utf8();
                self.word.add_replacement(start, end, Eszett(*casing));
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

    // Pretty much all ordinarily lowercase words *might* appear uppercased, e.g. at the
    // beginning of sentences. For example: "Uebel!" -> "Übel!", even though only "übel"
    // is in the dictionary.
    if first_char(word).is_uppercase() && is_valid(&lowercase_first_char(word), words) {
        trace!("Candidate '{}' is valid when lowercased.", word);
        return true;
    }

    let search = |word| words.binary_search(&word).is_ok();

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
            return is_valid(&uppercase_first_char(suffix), words);
        }

        trace!("Prefix not found in word list, trying next.");
    }

    false
}

impl TextProcessor for German {
    fn process(&self, input: &mut String) -> bool {
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
                } else {
                    trace!("Candidate is invalid word, trying the next one.");
                }

                candidate = get_fresh_candidate();
            }

            output.push_str(&candidate);

            // Add back the non-word character that caused the exit transition.
            output.push(char);
        }

        debug!("Final string is '{}'", output);
        *input = output;

        let c = input.pop();
        debug_assert!(
            c == Some(INDICATOR),
            "Processor removed trailing indicator byte."
        );

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

    #[test]
    #[should_panic]
    fn test_is_valid_panics_on_empty_input() {
        is_valid("", WORDS);
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
                insta::assert_yaml_snapshot!(data.to_string(), is_valid(&word, WORDS));
            }
        )
    }
}
