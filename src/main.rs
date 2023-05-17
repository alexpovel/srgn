use env_logger::Env;
use itertools::Itertools;
use log::{debug, info};
use std::{
    collections::HashSet,
    io::{stdin, Read},
};

const EXPECTABLE_MAXIMUM_WORD_LENGTH_BYTES: u8 = 64;
const EXPECTABLE_MAXIMUM_MATCHES_PER_WORD: u8 = 8;

#[derive(Debug)]
enum SpecialCharacter {
    Umlaut(Umlaut),
    Eszett,
}

#[derive(Debug, Clone, Copy)]
enum Umlaut {
    Ue,
    Oe,
    Ae,
}

// #[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Default, Debug)]
enum State {
    Word(Option<Potential>),
    #[default]
    Other,
    // Match,
}

#[derive(Debug)]
struct Span {
    start: usize,
    end: usize,
}

#[derive(Debug)]
struct Match {
    span: Span,
    content: SpecialCharacter,
}

#[derive(Debug)]
struct Potential(SpecialCharacter);

#[derive(Debug)]
enum Transition {
    Entered,
    Exited,
    Internal,
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

#[derive(Debug)]
struct Word {
    content: String,
    matches: Vec<Match>,
}

impl Word {
    /// Clears the word's contents while retaining any allocated capacities.
    fn clear(&mut self) {
        self.content.clear();
        self.matches.clear();
    }
}

impl Default for Word {
    fn default() -> Self {
        Self {
            content: String::with_capacity(EXPECTABLE_MAXIMUM_WORD_LENGTH_BYTES as usize),
            matches: Vec::with_capacity(EXPECTABLE_MAXIMUM_MATCHES_PER_WORD as usize),
        }
    }
}

struct Machine {
    state: State,
    word: Word,
    transition: Option<Transition>,
}

impl Machine {
    fn new() -> Self {
        Self {
            state: State::default(),
            word: Word::default(),
            transition: None,
        }
    }

    fn pre_transitition(&mut self) {
        if let State::Other = self.state {
            self.word.clear();
        };
    }

    fn post_transitition(&mut self, input: &MachineInput, next: &State) {
        self.transition = Some(Transition::from_states(&self.state, next));

        if let Some(Transition::Entered | Transition::Internal) = self.transition {
            self.word.content.push(*input);
        };
    }

    fn transition(&mut self, input: &MachineInput) -> &Option<Transition> {
        self.pre_transitition();

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
                let pos = self.word.content.len();
                let match_ = Match {
                    span: Span {
                        start: pos - c.len_utf8(),
                        end: pos + c.len_utf8(),
                    },
                    content: SpecialCharacter::Eszett,
                };

                self.word.matches.push(match_);
                State::Word(None)
            }
            (State::Word(Some(Potential(SpecialCharacter::Umlaut(umlaut)))), c @ 'e') => {
                let pos = self.word.content.len();
                let match_ = Match {
                    span: Span {
                        start: pos - 1,
                        end: pos + c.len_utf8(),
                    },
                    content: SpecialCharacter::Umlaut(*umlaut),
                };

                self.word.matches.push(match_);
                State::Word(None)
            }
            //
            (_, c) if c.is_alphabetic() => State::Word(None),
            (_, _) => State::Other,
        };

        self.post_transitition(input, &next);

        self.state = next;
        &self.transition
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    info!("Launching app...");

    let raw = include_str!("../de.txt");
    // let words: HashSet<&str> = HashSet::with_capacity(2 ^ 15);
    let _words: HashSet<&str> = HashSet::from_iter(raw.lines());
    let mut _output: Vec<char> = Vec::new();
    // let mut current_word = String::with_capacity(50);

    let mut inbuf = String::new();
    stdin().read_to_string(&mut inbuf).unwrap();

    debug!("Input buffer reads: {}", inbuf);

    let mut _outbuf = String::with_capacity(inbuf.capacity());

    let mut machine = Machine::new();

    // let mut previous = None;

    for char in inbuf.chars() {
        debug!("Beginning processing of character '{}'", char);

        let transition = machine.transition(&char);

        debug!("Transition is '{:?}'", transition);

        if let Some(Transition::Exited) = transition {
            debug!("Exited word: {:?}", machine.word);
            // let res = current_word.replace("ue", "ü");
            // _outbuf.push_str(&res);
            // current_word.clear();
        }

        // let x = machine;

        // .expect("FSM should be implemented such that no invalid transitions exist.");

        // debug!("Machine state is '{:?}'", state);

        // let Some(wt) = state else {
        //     _outbuf.push(char);
        //     continue;
        // };

        // current_word.push(char);
        // if let (WordTransition::Exited, Some(State::Match)) = (wt, previous) {
        //     let res = current_word.replace("ue", "ü");
        //     _outbuf.push_str(&res);
        //     current_word.clear();
        // }

        // previous = Some(*machine.state());
    }

    debug!("Final string is '{}'", _outbuf);
    info!("Exiting.");
}

// println!("{words:?}");
// let out = String::from_iter(output);
// println!("{out}");
// }

fn process(word: &str, words: HashSet<String>, possibilities: Vec<String>) {
    // possibilities = Vec::new();
    let mut current = String::new();
    for char in word.chars() {
        current.push(char);
        if words.contains(&current) {}
    }
}

fn power_set<T: Clone>(elements: Vec<T>) -> Vec<Vec<T>> {
    let mut result = Vec::new();
    for i in 1..=elements.len() {
        let subelements = elements.clone().into_iter().combinations(i);
        result.extend(subelements);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::power_set;
    use rstest::rstest;

    type TestVec = Vec<i32>;

    #[rstest]
    #[case(Vec::new(), vec![])]
    #[case(vec![1], vec![vec![1]])]
    #[case(vec![1, 2], vec![vec![1], vec![2], vec![1, 2]])]
    #[case(
        vec![1, 2, 3],
        vec![
            vec![1],
            vec![2],
            vec![3],
            vec![1, 2],
            vec![1, 3],
            vec![2, 3],
            vec![1, 2, 3]
        ]
    )]
    fn test_powerset_of_integers(#[case] input: TestVec, #[case] expected: Vec<TestVec>) {
        let result = power_set(input);
        assert_eq!(result, expected);
    }

    #[rstest]
    fn test_powerset_of_tuples() {
        let input = vec![(1, 2), (2, 4), (3, 9)];
        let expected = vec![
            vec![(1, 2)],
            vec![(2, 4)],
            vec![(3, 9)],
            vec![(1, 2), (2, 4)],
            vec![(1, 2), (3, 9)],
            vec![(2, 4), (3, 9)],
            vec![(1, 2), (2, 4), (3, 9)],
        ];

        let result = power_set(input);
        assert_eq!(result, expected);
    }
}
