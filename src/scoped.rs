use log::debug;
use regex::Regex;

use crate::GLOBAL_SCOPE;

/// A scope to apply a [`Stage`] to.
///
/// A scope is a newtype around a regular expression pattern, and used to split a given
/// string into [`ScopeStatus`]es. The scope can span any regex, including the entire
/// input (`.*`), or individual characters.
///
/// Special care should be given to greedy matching, which is the
/// [default](https://docs.rs/regex/latest/regex/#repetitions). It might extend to scope
/// further than intended.
#[derive(Debug, Clone)]
pub struct Scope(Regex);

impl Scope {
    /// Create a new [`Scope`].
    #[must_use]
    pub fn new(pattern: Regex) -> Self {
        Self(pattern)
    }
}

impl From<Regex> for Scope {
    fn from(r: Regex) -> Self {
        Self(r)
    }
}

impl From<Scope> for Regex {
    fn from(s: Scope) -> Self {
        s.0
    }
}

impl From<&Scope> for Regex {
    fn from(s: &Scope) -> Self {
        s.0.clone()
    }
}

impl From<&Regex> for Scope {
    fn from(r: &Regex) -> Self {
        Self(r.clone())
    }
}

impl Default for Scope {
    /// Create a new [`Scope`] that matches everything ([`GLOBAL_SCOPE`]).
    fn default() -> Self {
        Self(Regex::new(GLOBAL_SCOPE).unwrap())
    }
}

/// A trait for splitting a string into [`ScopeStatus`]es.
///
/// [`Stage`]s are [`Scoped`], such that their processing can be applied only to parts
/// in some [`Scope`] (these are [`InScope`]), and not to parts outside of it (these
/// are [`OutOfScope`]).
pub trait Scoped {
    /// Given some `input` and a corresponding [`Scope`], split the `input` into
    /// consecutive [`ScopeStatus`]es according to the `scope`.
    ///
    /// This is like [`Regex::find_iter`] (matched items are considered [`InScope`]),
    /// but also returns [`OutOfScope`] (i.e., unmatched) items, interleaved. As such,
    /// reassembling all returned [`str`] parts yields back the original `input`.
    ///
    /// The returned [`Vec`] does not necessarily contain alternatingly scoped slices.
    /// Multiple [`InScope`] items in a row might be returned if corresponding
    /// consecutive matches are found. However, [`OutOfScope`] items cannot follow one
    /// another directly. Empty [`str`] slices are not returned.
    ///
    // # Examples
    //
    // ```
    // use regex::Regex;
    // use text_processing_pipeline::scoped::{Scope, Scoped, ScopeStatus};
    // ```
    fn split_by_scope<'a>(&self, input: &'a str, scope: &Scope) -> Vec<ScopeStatus<'a>> {
        let mut scopes = Vec::new();
        let mut last_end = 0;

        for m in scope.0.find_iter(input) {
            if m.start() > last_end {
                scopes.push(ScopeStatus::OutOfScope(&input[last_end..m.start()]));
            }

            scopes.push(ScopeStatus::InScope(m.as_str()));
            last_end = m.end();
        }

        scopes.push(ScopeStatus::OutOfScope(&input[last_end..]));

        scopes.retain(|s| {
            let s: &str = s.into();
            !s.is_empty()
        });

        debug!("Scopes to work on: {:?}", scopes);
        scopes
    }
}

/// Indicates whether a given string part is in scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeStatus<'a> {
    /// The given string part is in scope for processing.
    InScope(&'a str),
    /// The given string part is out of scope for processing.
    OutOfScope(&'a str),
}

impl<'a> From<&'a ScopeStatus<'_>> for &'a str {
    /// Get the underlying string slice of a [`ScopeStatus`].
    ///
    /// All variants contain such a slice, so this is a convenient method.
    fn from(s: &'a ScopeStatus) -> Self {
        match s {
            ScopeStatus::InScope(s) | ScopeStatus::OutOfScope(s) => s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ScopeStatus::{InScope, OutOfScope};
    use super::*;
    use rstest::rstest;

    struct Dummy;
    impl Scoped for Dummy {}

    /// Run some manual testing for sanity. Random/fuzzing/property testing is much
    /// better in this case. See below.
    #[rstest]
    #[case("a", "", vec![OutOfScope("a")])]
    #[case("", "a", vec![])] // Empty results are discarded
    //
    #[case("a", "a", vec![InScope("a")])]
    #[case("a", "b", vec![OutOfScope("a")])]
    //
    #[case("a", ".*", vec![InScope("a")])]
    #[case("a", ".+?", vec![InScope("a")])]
    //
    #[case("a\na", ".*", vec![InScope("a"), OutOfScope("\n"), InScope("a")])]
    #[case("a\na", "(?s).*", vec![InScope("a\na")])] // Dot matches newline
    //
    #[case("abc", "a", vec![InScope("a"), OutOfScope("bc")])]
    //
    #[case("abc", r"\w", vec![InScope("a"), InScope("b"), InScope("c")])]
    #[case("abc", r"\W", vec![OutOfScope("abc")])]
    #[case("abc", r"\w+", vec![InScope("abc")])]
    //
    #[case("Work 69 on 420 words", r"\w+", vec![InScope("Work"), OutOfScope(" "), InScope("69"), OutOfScope(" "), InScope("on"), OutOfScope(" "), InScope("420"), OutOfScope(" "), InScope("words")])]
    #[case("Ignore 69 the 420 digits", r"\p{letter}+", vec![InScope("Ignore"), OutOfScope(" 69 "), InScope("the"), OutOfScope(" 420 "), InScope("digits")])]
    fn test_split_by_scope(
        #[case] input: &str,
        #[case] scope: &str,
        #[case] expected: Vec<ScopeStatus>,
    ) {
        let scope = Scope::from(Regex::new(scope).unwrap());
        let dummy = Dummy {};

        let scopes = dummy.split_by_scope(input, &scope);

        assert_eq!(scopes, expected);
    }

    mod random {
        use std::time::{Duration, Instant};

        use super::*;

        use log::info;
        use rand;
        use rand::seq::SliceRandom;
        use rand::Rng;
        use test_log::test;

        fn generate_random_regex(mut rng: &mut rand::rngs::ThreadRng) -> Option<Regex> {
            let atoms: [&str; 7] = [".", "\\d", "\\D", "\\w", "\\W", "\\s", "\\S"];
            let quantifiers: [&str; 5] = ["*", "+", "?", "{2,5}", "{3}"];
            let others: [&str; 3] = ["|", "^", "$"];
            let letters: [&str; 26] = [
                "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
                "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
            ];

            let mut regex = String::new();

            for _ in 0..rng.gen_range(1..=2) {
                if rng.gen_bool(0.3) {
                    regex.push_str(atoms.choose(&mut rng).unwrap());
                }

                if rng.gen_bool(0.6) {
                    let letter = letters.choose(&mut rng).unwrap();
                    if rng.gen_bool(0.5) {
                        let uc = letter.to_uppercase();
                        regex.push_str(uc.as_str());
                    } else {
                        regex.push_str(letter);
                    }
                }

                if rng.gen_bool(0.3) {
                    regex.push_str(quantifiers.choose(&mut rng).unwrap());
                }

                if rng.gen_bool(0.1) {
                    regex.push_str(others.choose(&mut rng).unwrap());
                }
            }

            Regex::new(regex.as_str()).ok()
        }

        /// Run fuzz-like testing.
        ///
        /// This is much like fuzzing, but a bit more manually controlled and part of
        /// the core test harness, hence running always. Property testing like
        /// `proptest` would be much better ("given some input in this shape, and some
        /// regex, test the property that reassembly works"), but setup for that crate
        /// is substantial. The below approach is 'good enough' and emulates property
        /// testing a fair bit. We just need some random inputs and some short-ish but
        /// random regex to split by (generating random, valid regex is... interesting).
        ///
        /// Run for a duration instead of a fixed number of tries, as we would have to
        /// choose that fixed number rather low for CI to not be too slow. That would
        /// waste potential when running locally.
        #[test]
        fn test_scoping_randomly() {
            let mut n_tries = 0;
            let mut n_matches = 0;

            let duration = if std::env::var("CI").is_ok() {
                Duration::from_secs(5)
            } else {
                // SORRY if this crashed the test on your machine. Flaky one :(
                Duration::from_millis(500)
            };

            let mut rng = rand::thread_rng();

            // "Anything but 'other'", see also:
            // https://docs.rs/regex/latest/regex/#matching-one-character
            // https://www.unicode.org/reports/tr44/tr44-24.html#General_Category_Values
            let pattern = r"\P{other}+";
            let gen = rand_regex::Regex::compile(pattern, 100).unwrap();

            let now = Instant::now();
            let dummy = Dummy {};

            loop {
                n_tries += 1;

                let Some(regex) = generate_random_regex(&mut rng) else {
                    continue;
                };
                let scope = Scope::from(regex);
                let input: String = rng.sample(&gen);

                let scopes = dummy.split_by_scope(&input, &scope);

                if scopes.iter().any(|s| match s {
                    InScope(_) => true,
                    OutOfScope(_) => false,
                }) {
                    n_matches += 1;
                }

                let mut reassembled = String::new();
                for scope in scopes {
                    reassembled.push_str((&scope).into());
                }

                assert_eq!(input, reassembled);

                if now.elapsed() > duration {
                    break;
                }
            }

            info!(
                // To test anything, we actually need matches so splits happen.
                "Processed {} inputs, of which {} were matched and successfully reassembled",
                n_tries, n_matches
            );

            assert!(
                n_matches >= n_tries / 20,
                "Too few regex matches; try lowering regex length"
            );

            assert!(
                n_tries > 250,
                // Might happen in CI, but we should ensure a certain lower bound;
                // locally, many more tests can run.
                "Too few tries; is the host machine very slow?"
            );
        }
    }
}
