/// A dummy implementation of the Circuit Breaker pattern to demonstrate
/// capabilities of this library.
/// https://martinfowler.com/bliki/CircuitBreaker.html
use rust_fsm::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug)]
enum CircuitBreakerInput {
    Successful,
    Unsuccessful,
    TimerTriggered,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, PartialEq)]
struct CircuitBreakerOutputSetTimer;

#[derive(Debug)]
struct CircuitBreakerMachine;

impl StateMachineImpl for CircuitBreakerMachine {
    type Input = CircuitBreakerInput;
    type State = CircuitBreakerState;
    type Output = CircuitBreakerOutputSetTimer;
    const INITIAL_STATE: Self::State = CircuitBreakerState::Closed;

    fn transition(state: &Self::State, input: &Self::Input) -> Option<Self::State> {
        match (state, input) {
            (CircuitBreakerState::Closed, CircuitBreakerInput::Unsuccessful) => {
                Some(CircuitBreakerState::Open)
            }
            (CircuitBreakerState::Open, CircuitBreakerInput::TimerTriggered) => {
                Some(CircuitBreakerState::HalfOpen)
            }
            (CircuitBreakerState::HalfOpen, CircuitBreakerInput::Successful) => {
                Some(CircuitBreakerState::Closed)
            }
            (CircuitBreakerState::HalfOpen, CircuitBreakerInput::Unsuccessful) => {
                Some(CircuitBreakerState::Open)
            }
            _ => None,
        }
    }

    fn output(state: &Self::State, input: &Self::Input) -> Option<Self::Output> {
        match (state, input) {
            (CircuitBreakerState::Closed, CircuitBreakerInput::Unsuccessful) => {
                Some(CircuitBreakerOutputSetTimer)
            }
            (CircuitBreakerState::HalfOpen, CircuitBreakerInput::Unsuccessful) => {
                Some(CircuitBreakerOutputSetTimer)
            }
            _ => None,
        }
    }
}

#[test]
fn circuit_breaker() {
    let machine: StateMachine<CircuitBreakerMachine> = StateMachine::new();

    // Unsuccessful request
    let machine = Arc::new(Mutex::new(machine));
    {
        let mut lock = machine.lock().unwrap();
        let res = lock.consume(&CircuitBreakerInput::Unsuccessful).unwrap();
        assert_eq!(res, Some(CircuitBreakerOutputSetTimer));
        assert_eq!(lock.state(), &CircuitBreakerState::Open);
    }

    // Set up a timer
    let machine_wait = machine.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::new(5, 0));
        let mut lock = machine_wait.lock().unwrap();
        let res = lock.consume(&CircuitBreakerInput::TimerTriggered).unwrap();
        assert_eq!(res, None);
        assert_eq!(lock.state(), &CircuitBreakerState::HalfOpen);
    });

    // Try to pass a request when the circuit breaker is still open
    let machine_try = machine.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::new(1, 0));
        let mut lock = machine_try.lock().unwrap();
        let res = lock.consume(&CircuitBreakerInput::Successful);
        assert!(matches!(res, Err(TransitionImpossibleError)));
        assert_eq!(lock.state(), &CircuitBreakerState::Open);
    });

    // Test if the circit breaker was actually closed
    std::thread::sleep(Duration::new(7, 0));
    {
        let mut lock = machine.lock().unwrap();
        let res = lock.consume(&CircuitBreakerInput::Successful).unwrap();
        assert_eq!(res, None);
        assert_eq!(lock.state(), &CircuitBreakerState::Closed);
    }
}
