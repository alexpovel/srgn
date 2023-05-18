use itertools::Itertools;

use crate::{iteration::power_set, modules::TextProcessor};

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

                let start = pos - c.len_utf8();
                let end = pos + c.len_utf8();
                self.word.add_match(start, end, SpecialCharacter::Eszett);
                State::Word(None)
            }
            (State::Word(Some(Potential(SpecialCharacter::Umlaut(umlaut)))), c @ 'e') => {
                let pos = self.word.len();

                let start = pos - 1;
                let end = pos + c.len_utf8();
                self.word
                    .add_match(start, end, SpecialCharacter::Umlaut(*umlaut));
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

struct ModuleImpl;

impl TextProcessor for ModuleImpl {
    fn process(&self, input: &mut String) -> bool {
        // debug!("Input buffer reads: {}", inbuf);

        let mut output = String::with_capacity(input.capacity());

        let mut machine = StateMachine::new();

        for char in input.chars() {
            // debug!("Beginning processing of character '{}'", char);

            let transition = machine.transition(&char);

            // debug!("Transition is '{:?}'", transition);

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

            // debug!("Exited word: {:?}", machine.word);

            let match_sets = power_set(machine.word.matches().clone().into_iter());
            // debug!("All matches: {:?}", match_sets);

            let get_fresh_candidate = || machine.word.content().clone();
            let mut candidate = get_fresh_candidate();

            for match_set in match_sets {
                let iter = match_set.iter().rev();

                // We are replacing starting from behind, such that earlier indices are not
                // invalidated.
                debug_assert!(iter
                    .clone()
                    .collect_vec()
                    .windows(2)
                    .all(|tuple| tuple[0].start() > tuple[1].start()));

                for match_ in iter {
                    let replacement = match_.content().value();

                    candidate.replace_range(match_.start()..match_.end(), &replacement);
                    // debug!("Replaced candidate word, now is: {}", candidate);
                }

                // if words.contains(&candidate as &str) {
                //     break;
                // }

                candidate = get_fresh_candidate();
            }

            output.push_str(&candidate);

            // Add back the non-word character that caused the exit transition.
            output.push(char);
        }

        // debug!("Final string is '{}'", outbuf);
        // inbuf
        true
    }
}
