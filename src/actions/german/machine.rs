use log::trace;

use super::LetterCasing::{Lower, Upper};
use super::SpecialCharacter::{Eszett, Umlaut};
use super::Umlaut::{Ae, Oe, Ue};
use super::{SpecialCharacter, Word};

#[derive(Default, Debug)]
enum State {
    #[default]
    Other,
    Word(Option<Potential>),
}

/// This is basically just an `Option`, but it's more descriptive and leads to really
/// nicely readable code (an avoids `Option<Option<T>>`, which could be confusing/less
/// readable).
#[derive(Debug)]
struct Potential(SpecialCharacter);

#[derive(Debug, Clone)]
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
    const fn from_states(from: &State, to: &State) -> Self {
        match (from, to) {
            (State::Word(_), State::Other) => Self::Exited,
            (State::Other, State::Word(_)) => Self::Entered,
            (State::Word(_), State::Word(_)) => Self::Internal,
            (State::Other, State::Other) => Self::External,
        }
    }
}

type MachineInput = char;

#[derive(Debug)]
pub(super) struct StateMachine {
    state: State,
    word: Word,
    transition: Option<Transition>,
}

impl StateMachine {
    pub(super) fn new() -> Self {
        Self {
            state: State::default(),
            word: Word::default(),
            transition: None,
        }
    }

    pub(super) const fn current_word(&self) -> &Word {
        &self.word
    }

    fn pre_transition(&mut self) {
        if matches!(self.state, State::Other) {
            self.word.clear();

            trace!("Cleared current word, machine now is: {self:?}.");
        }
    }

    pub(super) fn transition(&mut self, input: MachineInput) -> Transition {
        self.pre_transition();

        let next = match (&self.state, input) {
            (State::Word(Some(Potential(Umlaut(umlaut)))), c @ ('e' | 'E')) => {
                const LENGTH_OF_PREVIOUS_CHARACTER: usize = 1;

                let pos = self.word.len();

                // We're in a state machine, so we cannot know the length of the
                // previous character, as have to assume its length here.
                debug_assert!(
                    'o'.len_utf8() == LENGTH_OF_PREVIOUS_CHARACTER
                        && 'u'.len_utf8() == LENGTH_OF_PREVIOUS_CHARACTER
                        && 'a'.len_utf8() == LENGTH_OF_PREVIOUS_CHARACTER
                );

                let start = pos - LENGTH_OF_PREVIOUS_CHARACTER;
                let end = pos + c.len_utf8();
                self.word.add_replacement(start, end, Umlaut(*umlaut));

                trace!("Added replacement at position {pos}, machine now is: {self:?}.");

                State::Word(None)
            }
            (State::Word(Some(Potential(Eszett(casing)))), c @ ('s' | 'S')) => {
                let pos = self.word.len();

                let start = pos - c.len_utf8(); // Previous char same as current `c`
                let end = pos + c.len_utf8();
                self.word.add_replacement(start, end, Eszett(*casing));

                trace!("Added replacement at position {pos}, machine now is: {self:?}.");

                State::Word(None)
            }
            (_, 'a') => State::Word(Some(Potential(Umlaut(Ae(Lower))))),
            (_, 'A') => State::Word(Some(Potential(Umlaut(Ae(Upper))))),
            (_, 'o') => State::Word(Some(Potential(Umlaut(Oe(Lower))))),
            (_, 'O') => State::Word(Some(Potential(Umlaut(Oe(Upper))))),
            (_, 'u') => State::Word(Some(Potential(Umlaut(Ue(Lower))))),
            (_, 'U') => State::Word(Some(Potential(Umlaut(Ue(Upper))))),
            (_, 's') => State::Word(Some(Potential(Eszett(Lower)))),
            (_, 'S') => State::Word(Some(Potential(Eszett(Upper)))),
            (_, c) if c.is_alphabetic() => State::Word(None),
            _ => State::Other,
        };

        let transition = Transition::from_states(&self.state, &next);

        self.state = next;
        self.transition = Some(transition.clone()); // Clone, else it gets awkward returning.

        self.post_transition(input);

        transition
    }

    fn post_transition(&mut self, input: MachineInput) {
        if let Some(Transition::Entered | Transition::Internal) = self.transition {
            self.word.push(input);
            trace!(
                "Appending {:?} to current word due to transition {:?}.",
                input, self.transition
            );
        }

        trace!("After transition, machine is: {self:?}.");
    }
}
