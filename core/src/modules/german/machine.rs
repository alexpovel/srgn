use super::{
    Casing::Lower, Casing::Upper, SpecialCharacter, SpecialCharacter::Eszett,
    SpecialCharacter::Umlaut, Umlaut::Ae, Umlaut::Oe, Umlaut::Ue, Word,
};

use log::trace;

#[derive(Default, Debug)]
enum State {
    #[default]
    Other,
    Word(Option<Potential>),
}

#[derive(Debug)]
struct Potential(SpecialCharacter);

#[derive(Debug)]
pub(super) enum Transition {
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

pub(super) struct StateMachine {
    state: State,
    word: Word,
    transition: Option<Transition>,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: State::default(),
            word: Word::default(),
            transition: None,
        }
    }

    pub fn current_word(&self) -> &Word {
        &self.word
    }

    fn pre_transition(&mut self) {
        if let State::Other = self.state {
            trace!(
                "In state '{:?}', clearing current word '{:?}'.",
                self.state,
                self.word
            );
            self.word.clear();
        };
    }

    pub fn transition(&mut self, input: &MachineInput) -> &Option<Transition> {
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

                trace!(
                    "In state '{:?}', added replacement '{:?}' at position {}.",
                    self.state,
                    self.word.replacements().last().unwrap(),
                    pos
                );

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

                trace!(
                    "In state '{:?}', added replacement '{:?}' at position {}.",
                    self.state,
                    self.word.replacements().last().unwrap(),
                    pos
                );

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
            trace!(
                "In state '{:?}', building up current word '{:?}' with character '{:?}' due to transition {:?}",
                self.state,
                self.word,
                input,
                self.transition
            );

            self.word.push(*input);
        };
    }
}
