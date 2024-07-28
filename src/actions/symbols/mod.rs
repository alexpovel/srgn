#[cfg(all(doc, feature = "german"))]
use super::German;
use crate::actions::Action;
#[cfg(test)]
use enum_iterator::{all, Sequence};
use std::collections::VecDeque;

pub mod inversion;

/// Replace ASCII symbols (`--`, `->`, `!=`, ...) with proper Unicode equivalents (`–`,
/// `→`, `≠`, ...).
///
/// This action is greedy, i.e. it will try to replace as many symbols as possible,
/// replacing left-to-right as greedily as possible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Symbols {}

macro_rules! fetch_next {
    ($it:expr, $stack:expr, $buf:expr $(, $label:tt)?) => {
        if let Some(c) = $it.pop_front() {
            $stack.push(c);
            c
        } else {
            $buf.push_str(&$stack.into_iter().collect::<String>());

            // Control flow, thus a macro is required. Optionally, allow a label for
            // more control, e.g. when looping while waiting.
            break $($label)?;
        }
    };
}

impl Action for Symbols {
    /// ## Implementation note
    ///
    /// Only relevant when looking at the source code.
    ///
    /// The implementation is in the style of coroutines as presented [in this
    /// article](https://www.chiark.greenend.org.uk/~sgtatham/quasiblog/coroutines-philosophy/).
    /// Instead of constructing an explicit state machine (like in [`German`]), we
    /// use a generator coroutine to consume values from. The position in code itself is
    /// then our state. `undo_overfetching` is a bit like sending a value back into the
    /// coroutine so it can be yielded again.
    ///
    /// All in all, ugly and verbose, would not recommend, but a worthwhile experiment.
    #[allow(clippy::cognitive_complexity)] // Yep, it's terrible alright
    fn act(&self, input: &str) -> String {
        let mut deque = input.chars().collect::<VecDeque<_>>();
        let mut out = String::new();

        'outer: loop {
            let mut stack = Vec::new();

            match fetch_next!(deque, stack, out) {
                '-' => match fetch_next!(deque, stack, out) {
                    '-' => {
                        // Be greedy, could be last character
                        replace(&mut stack, Symbol::EnDash);

                        match fetch_next!(deque, stack, out) {
                            '-' => replace(&mut stack, Symbol::EmDash),
                            '>' => replace(&mut stack, Symbol::LongRightArrow),
                            _ => undo_overfetching(&mut deque, &mut stack),
                        }
                    }
                    '>' => replace(&mut stack, Symbol::ShortRightArrow),
                    _ => undo_overfetching(&mut deque, &mut stack),
                },
                '<' => match fetch_next!(deque, stack, out) {
                    '-' => {
                        // Be greedy, could be last character
                        replace(&mut stack, Symbol::ShortLeftArrow);

                        match fetch_next!(deque, stack, out) {
                            '-' => replace(&mut stack, Symbol::LongLeftArrow),
                            '>' => replace(&mut stack, Symbol::LeftRightArrow),
                            _ => undo_overfetching(&mut deque, &mut stack),
                        }
                    }
                    '=' => replace(&mut stack, Symbol::LessThanOrEqual),
                    _ => undo_overfetching(&mut deque, &mut stack),
                },
                '>' => match fetch_next!(deque, stack, out) {
                    '=' => replace(&mut stack, Symbol::GreaterThanOrEqual),
                    _ => undo_overfetching(&mut deque, &mut stack),
                },
                '!' => match fetch_next!(deque, stack, out) {
                    '=' => replace(&mut stack, Symbol::NotEqual),
                    _ => undo_overfetching(&mut deque, &mut stack),
                },
                '=' => match fetch_next!(deque, stack, out) {
                    '>' => replace(&mut stack, Symbol::RightDoubleArrow),
                    _ => undo_overfetching(&mut deque, &mut stack),
                },
                // "Your scientists were so preoccupied with whether or not they could,
                // they didn't stop to think if they should." ... this falls into the
                // "shouldn't" category:
                'h' => match fetch_next!(deque, stack, out) {
                    't' => match fetch_next!(deque, stack, out) {
                        't' => match fetch_next!(deque, stack, out) {
                            'p' => match fetch_next!(deque, stack, out) {
                                // Sorry, `http` not supported. Neither is `ftp`,
                                // `file`, ...
                                's' => match fetch_next!(deque, stack, out) {
                                    ':' => match fetch_next!(deque, stack, out) {
                                        '/' => match fetch_next!(deque, stack, out) {
                                            '/' => loop {
                                                match fetch_next!(deque, stack, out, 'outer) {
                                                    ' ' | '"' => break,
                                                    _ => {
                                                        // building up stack, ignoring
                                                        // all characters other than
                                                        // non-URI ones
                                                    }
                                                }
                                            },
                                            _ => undo_overfetching(&mut deque, &mut stack),
                                        },
                                        _ => undo_overfetching(&mut deque, &mut stack),
                                    },
                                    _ => undo_overfetching(&mut deque, &mut stack),
                                },
                                _ => undo_overfetching(&mut deque, &mut stack),
                            },
                            _ => undo_overfetching(&mut deque, &mut stack),
                        },
                        _ => undo_overfetching(&mut deque, &mut stack),
                    },
                    _ => undo_overfetching(&mut deque, &mut stack),
                },
                _ => {}
            }

            out.push_str(&stack.into_iter().collect::<String>());
        }

        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(Sequence))]
enum Symbol {
    // Typographic symbols
    EmDash,
    EnDash,
    // Arrows
    ShortRightArrow,
    ShortLeftArrow,
    LongRightArrow,
    LongLeftArrow,
    LeftRightArrow,
    RightDoubleArrow,
    // Math
    NotEqual,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

impl From<Symbol> for char {
    fn from(symbol: Symbol) -> Self {
        match symbol {
            Symbol::EnDash => '–',
            Symbol::EmDash => '—',
            //
            Symbol::ShortRightArrow => '→',
            Symbol::ShortLeftArrow => '←',
            Symbol::LongRightArrow => '⟶',
            Symbol::LongLeftArrow => '⟵',
            Symbol::LeftRightArrow => '↔',
            Symbol::RightDoubleArrow => '⇒',
            //
            Symbol::NotEqual => '≠',
            Symbol::LessThanOrEqual => '≤',
            Symbol::GreaterThanOrEqual => '≥',
        }
    }
}

impl TryFrom<char> for Symbol {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            // Typographic symbols
            '–' => Ok(Self::EnDash),
            '—' => Ok(Self::EmDash),
            // Arrows
            '→' => Ok(Self::ShortRightArrow),
            '←' => Ok(Self::ShortLeftArrow),
            '⟶' => Ok(Self::LongRightArrow),
            '⟵' => Ok(Self::LongLeftArrow),
            '↔' => Ok(Self::LeftRightArrow),
            '⇒' => Ok(Self::RightDoubleArrow),
            // Math
            '≠' => Ok(Self::NotEqual),
            '≤' => Ok(Self::LessThanOrEqual),
            '≥' => Ok(Self::GreaterThanOrEqual),
            _ => Err(()),
        }
    }
}

/// We might greedily overfetch and then end up with a [`char`] on the `stack` we do not
/// know how to handle. However, *subsequent, other states might*. Hence, be a good
/// citizen and put it back where it came from.
///
/// This allows matching sequences like `--!=` to be `–≠`, which might otherwise end up
/// as `–!=` (because the next iteration only sees `=`, `!` was already consumed).
fn undo_overfetching<T>(deque: &mut VecDeque<T>, stack: &mut Vec<T>) {
    deque.push_front(
        stack
            .pop()
            .expect("Pop should only happen after having just pushed, so stack shouldn't be empty"),
    );
}

/// Replace the entire `stack` with the given `symbol`.
fn replace(stack: &mut Vec<char>, symbol: Symbol) {
    stack.clear();
    stack.push(symbol.into());
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("", "")]
    #[case(" ", " ")]
    // Typographic symbols
    #[case("--", "–")]
    #[case("---", "—")]
    // Arrows
    #[case("->", "→")]
    #[case("-->", "⟶")]
    #[case("<-", "←")]
    #[case("<--", "⟵")]
    #[case("<->", "↔")]
    #[case("=>", "⇒")]
    // Math
    #[case("<=", "≤")]
    #[case(">=", "≥")]
    #[case("!=", "≠")]
    fn test_symbol_substitution_base_cases(#[case] input: &str, #[case] expected: &str) {
        let action = Symbols::default();
        let result = action.act(input);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("A-", "A-")]
    #[case("A--", "A–")]
    #[case("A---", "A—")]
    //
    #[case("-A", "-A")]
    #[case("--A", "–A")]
    #[case("---A", "—A")]
    //
    #[case("A->", "A→")]
    #[case("A-->", "A⟶")]
    #[case("A<->", "A↔")]
    #[case("A=>", "A⇒")]
    //
    #[case("<-A", "←A")]
    #[case("<--A", "⟵A")]
    #[case("<->A", "↔A")]
    #[case("=>A", "⇒A")]
    //
    #[case("A<=", "A≤")]
    #[case("A>=", "A≥")]
    #[case("A!=", "A≠")]
    fn test_symbol_substitution_neighboring_single_letter(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        let action = Symbols::default();
        let result = action.act(input);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("A-B", "A-B")]
    #[case("A--B", "A–B")]
    #[case("A---B", "A—B")]
    //
    #[case("A->B", "A→B")]
    #[case("A-->B", "A⟶B")]
    #[case("A<->B", "A↔B")]
    #[case("A=>B", "A⇒B")]
    #[case("A<-B", "A←B")]
    #[case("A<--B", "A⟵B")]
    #[case("A<->B", "A↔B")]
    #[case("A=>B", "A⇒B")]
    //
    #[case("A<=B", "A≤B")]
    #[case("A>=B", "A≥B")]
    #[case("A!=B", "A≠B")]
    fn test_symbol_substitution_neighboring_letters(#[case] input: &str, #[case] expected: &str) {
        let action = Symbols::default();
        let result = action.act(input);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("A - B", "A - B")]
    #[case("A -- B", "A – B")]
    #[case("A --- B", "A — B")]
    //
    #[case("A -> B", "A → B")]
    #[case("A --> B", "A ⟶ B")]
    #[case("A <-> B", "A ↔ B")]
    #[case("A => B", "A ⇒ B")]
    #[case("A <- B", "A ← B")]
    #[case("A <-- B", "A ⟵ B")]
    #[case("A <-> B", "A ↔ B")]
    #[case("A => B", "A ⇒ B")]
    //
    #[case("A <= B", "A ≤ B")]
    #[case("A >= B", "A ≥ B")]
    #[case("A != B", "A ≠ B")]
    fn test_symbol_substitution_neighboring_letters_with_spaces(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        let action = Symbols::default();
        let result = action.act(input);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("-X-", "-X-")]
    #[case("--X--", "–X–")]
    #[case("---X---", "—X—")]
    //
    #[case("-X>", "-X>")]
    #[case("->X->", "→X→")]
    #[case("--X-->", "–X⟶")]
    #[case("---X-->", "—X⟶")]
    //
    #[case("<-X-", "←X-")]
    #[case("<--X--", "⟵X–")]
    //
    #[case("<--X-->", "⟵X⟶")]
    fn test_symbol_substitution_disrupting_symbols(#[case] input: &str, #[case] expected: &str) {
        let action = Symbols::default();
        let result = action.act(input);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("I breathe -- I live.", "I breathe – I live.")]
    #[case("To think---to breathe.", "To think—to breathe.")]
    #[case("A joke --> A laugh.", "A joke ⟶ A laugh.")]
    #[case("A <= B => C", "A ≤ B ⇒ C")]
    #[case("->In->Out->", "→In→Out→")]
    fn test_symbol_substitution_sentences(#[case] input: &str, #[case] expected: &str) {
        let action = Symbols::default();
        let result = action.act(input);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("----", "—-")]
    #[case("-----", "—–")]
    #[case("------", "——")]
    //
    #[case(">->", ">→")]
    #[case("->->", "→→")]
    #[case("->-->", "→⟶")]
    #[case("->--->", "→—>")]
    #[case("->--->->", "→—>→")]
    //
    #[case("<-<-", "←←")]
    #[case("<-<--", "←⟵")]
    #[case("<-<---", "←⟵-")]
    #[case("<-<---<", "←⟵-<")]
    //
    #[case("<->->", "↔→")]
    #[case("<-<->->", "←↔→")]
    //
    #[case("<=<=", "≤≤")]
    #[case("<=<=<=", "≤≤≤")]
    #[case(">=>=", "≥≥")]
    #[case(">=>=>=", "≥≥≥")]
    //
    #[case(">=<=", "≥≤")]
    #[case(">=<=<=", "≥≤≤")]
    //
    #[case("!=!=", "≠≠")]
    #[case("!=!=!=", "≠≠≠")]
    fn test_symbol_substitution_ambiguous_sequences(#[case] input: &str, #[case] expected: &str) {
        let action = Symbols::default();
        let result = action.act(input);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("–", "–")]
    #[case("—", "—")]
    #[case("→", "→")]
    #[case("←", "←")]
    #[case("⟶", "⟶")]
    #[case("⟵", "⟵")]
    #[case("↔", "↔")]
    #[case("⇒", "⇒")]
    #[case("≠", "≠")]
    #[case("≤", "≤")]
    #[case("≥", "≥")]
    fn test_symbol_substitution_existing_symbol(#[case] input: &str, #[case] expected: &str) {
        let action = Symbols::default();
        let result = action.act(input);

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("https://www.example.com", "https://www.example.com")]
    #[case("https://www.example.com/", "https://www.example.com/")]
    #[case("https://www.example.com/->", "https://www.example.com/->")]
    //
    #[case("\"https://www.example.com/\"->", "\"https://www.example.com/\"→")]
    #[case("https://www.example.com/ ->", "https://www.example.com/ →")]
    //
    #[case("h->", "h→")]
    #[case("ht->", "ht→")]
    #[case("htt->", "htt→")]
    #[case("http->", "http→")]
    #[case("https->", "https→")]
    #[case("https:->", "https:→")]
    #[case("https:/->", "https:/→")]
    #[case("https://->", "https://->")] // Pivot point
    fn test_symbol_substitution_uri(#[case] input: &str, #[case] expected: &str) {
        let action = Symbols::default();
        let result = action.act(input);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_symbol_to_char_and_back_is_bijective() {
        let symbols: Vec<_> = all::<Symbol>().collect();

        for symbol in symbols {
            let c = char::from(symbol);
            let back = Symbol::try_from(c).expect("Should be able to convert back to symbol");

            assert_eq!(symbol, back);
        }
    }
}
