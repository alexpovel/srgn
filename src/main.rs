use env_logger::Env;
use itertools::Itertools;
use rstest::rstest;
// use itertools::Itertools;
use log::{debug, info};
use std::{
    collections::HashSet,
    io::{stdin, Read},
};

use rust_fsm::{StateMachine, StateMachineImpl};

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Initial,
    Regular(Option<Potential>),
    Other,
    Match,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Potential {
    Eszett,
    Umlaut,
}

#[derive(Debug)]
enum WordTransition {
    Internal,
    Entered,
    Exited,
}

struct Machine {}

impl StateMachineImpl for Machine {
    type Input = char;
    type State = State;
    type Output = WordTransition;
    const INITIAL_STATE: State = State::Initial;

    fn transition(state: &Self::State, input: &Self::Input) -> Option<Self::State> {
        let next = match (state, input) {
            (
                State::Initial
                | State::Regular(None)
                | State::Regular(Some(Potential::Umlaut))
                | State::Other,
                'o' | 'u' | 'a',
            ) => State::Regular(Some(Potential::Umlaut)),
            (State::Initial | State::Regular(None) | State::Other, 's') => {
                State::Regular(Some(Potential::Eszett))
            }
            //
            (State::Regular(Some(Potential::Umlaut)), 'e') => State::Match,
            (State::Regular(Some(Potential::Eszett)), 's') => State::Match,
            //
            (State::Match, c) if c.is_whitespace() => State::Other,
            (State::Match, _) => State::Match,
            //
            (_, c) if c.is_alphabetic() => State::Regular(None),
            (_, _) => State::Other,
        };

        Some(next)
    }

    fn output(state: &Self::State, input: &Self::Input) -> Option<Self::Output> {
        let next: State = Self::transition(state, input)?;

        match (state, next) {
            (State::Initial | State::Other, State::Regular(_)) => Some(WordTransition::Entered),
            (State::Regular(_), State::Regular(_)) => Some(WordTransition::Internal),
            (State::Match, State::Match) => Some(WordTransition::Internal),
            (State::Regular(Some(_)), State::Match) => Some(WordTransition::Internal),
            (State::Regular(_) | State::Match, State::Other) => Some(WordTransition::Exited),
            //
            (State::Initial | State::Other, State::Other) => None,
            //
            (State::Initial, State::Match) => panic!("Cannot match directly from initialization."),
            (State::Regular(_) | State::Other | State::Match | State::Initial, State::Initial) => {
                panic!("Cannot revert back to initial state.")
            }
            (State::Regular(None) | State::Other, State::Match) => {
                panic!("Cannot reach match immediately.")
            }
            (State::Match, State::Regular(_)) => {
                panic!("Cannot leave match unless it's a non-word.")
            }
        }
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    info!("Launching app...");

    let raw = include_str!("../de.txt");
    // let words: HashSet<&str> = HashSet::with_capacity(2 ^ 15);
    let _words: HashSet<&str> = HashSet::from_iter(raw.lines());
    let mut _output: Vec<char> = Vec::new();
    let mut current_word = String::with_capacity(50);

    let mut inbuf = String::new();
    stdin().read_to_string(&mut inbuf).unwrap();

    debug!("Input buffer reads: {}", inbuf);

    let mut _outbuf = String::with_capacity(inbuf.capacity());

    let mut machine: StateMachine<Machine> = StateMachine::new();

    let mut previous = None;

    for char in inbuf.chars() {
        debug!("Beginning processing of character '{}'", char);

        let transition = machine
            .consume(&char)
            .expect("FSM should be implemented such that no invalid transitions exist.");

        debug!(
            "Machine state is '{:?}', machine transition was '{:?}'",
            machine.state(),
            transition
        );

        let Some(wt) = transition else {
            _outbuf.push(char);
            continue;
        };

        current_word.push(char);
        if let (WordTransition::Exited, Some(State::Match)) = (wt, previous) {
            let res = current_word.replace("ue", "ü");
            _outbuf.push_str(&res);
            current_word.clear();
        }

        previous = Some(*machine.state());
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
    for i in 0..=elements.len() {
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
    #[case(Vec::new(), vec![Vec::new()])]
    #[case(vec![1], vec![Vec::new(), vec![1]])]
    #[case(vec![1, 2], vec![Vec::new(), vec![1], vec![2], vec![1, 2]])]
    #[case(
        vec![1, 2, 3],
        vec![
            Vec::new(),
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
            Vec::new(),
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
