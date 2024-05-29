use super::ROScopes;
use super::Scoper;
use crate::ranges::Ranges;
use crate::RegexPattern;
use crate::GLOBAL_SCOPE;
use log::{debug, trace};
use std::error::Error;
use std::fmt;

/// A regular expression for querying.
#[derive(Debug)]
pub struct Regex {
    pattern: RegexPattern,
}

impl Regex {
    /// Create a new regular expression.
    #[must_use]
    pub fn new(pattern: RegexPattern) -> Self {
        Self { pattern }
    }
}

/// An error that can occur when parsing a regular expression.
///
/// Simple wrapper.
#[derive(Debug)]
pub struct RegexError(fancy_regex::Error);

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid regex: {}", self.0)
    }
}

impl Error for RegexError {}

impl TryFrom<String> for Regex {
    type Error = RegexError;

    fn try_from(pattern: String) -> Result<Self, Self::Error> {
        let pattern = RegexPattern::new(&pattern).map_err(RegexError)?;

        Ok(Self::new(pattern))
    }
}

impl Default for Regex {
    fn default() -> Self {
        Self::new(RegexPattern::new(GLOBAL_SCOPE).unwrap())
    }
}

impl Scoper for Regex {
    fn scope<'viewee>(&self, input: &'viewee str) -> ROScopes<'viewee> {
        let has_capture_groups = self.pattern.captures_len() > 1;

        let ranges = if has_capture_groups {
            trace!(
                "Pattern '{}' has capture groups, iterating over matches",
                self.pattern
            );
            let mut ranges = Vec::new();
            for cap in self.pattern.captures_iter(input).flatten() {
                let mut it = cap.iter();

                let overall_match = it
                    .next()
                    // https://docs.rs/regex/1.9.5/regex/struct.SubCaptureMatches.html
                    .expect("Entered iterator of matches, but zeroth (whole) match missing")
                    .expect("First element guaranteed to be non-None (whole match)");
                trace!(
                    "Overall match: '{}' from index {} to {}",
                    overall_match.as_str().escape_debug(),
                    overall_match.start(),
                    overall_match.end()
                );

                let subranges = it.flatten().map(|m| m.range()).collect::<Ranges<_>>();
                trace!("Capture groups: {:?}", subranges);

                // Treat the capture groups specially
                subranges
                    .iter()
                    .for_each(|subrange| ranges.extend(Ranges::from(subrange)));

                // Parts of the overall match, but not the capture groups: push as-is
                ranges.extend(Ranges::from_iter([overall_match.range()]) - subranges);
            }

            let res = ranges.into_iter().collect();
            debug!("Ranges to scope after regex: {:?}", res);
            res
        } else {
            trace!(
                "No capture groups in pattern '{}', short-circuiting",
                input.escape_debug()
            );

            self.pattern
                .find_iter(input)
                .flatten()
                .map(|m| m.range())
                .collect()
        };

        ROScopes::from_raw_ranges(input, ranges)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::scoping::{
        scope::{
            RWScope, RWScopes,
            Scope::{In, Out},
        },
        view::ScopedView,
    };
    use std::borrow::Cow::Borrowed as B;

    use super::*;

    #[rstest]
    #[case("a", "a", ScopedView::new(RWScopes(vec![RWScope(In(B("a")))])))]
    #[case("aa", "a", ScopedView::new(RWScopes(vec![RWScope(In(B("a"))), RWScope(In(B("a")))])))]
    #[case("aba", "a", ScopedView::new(RWScopes(vec![RWScope(In(B("a"))), RWScope(Out("b")), RWScope(In(B("a")))])))]
    //
    #[case("a", "", ScopedView::new(RWScopes(vec![RWScope(Out("a"))])))]
    #[case("", "a", ScopedView::new(RWScopes(vec![])))] // Empty results are discarded
    //
    #[case("a", "a", ScopedView::new(RWScopes(vec![RWScope(In(B("a")))])))]
    #[case("a", "b", ScopedView::new(RWScopes(vec![RWScope(Out("a"))])))]
    //
    #[case("a", ".*", ScopedView::new(RWScopes(vec![RWScope(In(B("a")))])))]
    #[case("a", ".+?", ScopedView::new(RWScopes(vec![RWScope(In(B("a")))])))]
    //
    #[case("a\na", ".*", ScopedView::new(RWScopes(vec![RWScope(In(B("a"))), RWScope(Out("\n")), RWScope(In(B("a")))])))]
    #[case("a\na", "(?s).*", ScopedView::new(RWScopes(vec![RWScope(In(B("a\na")))])))] // Dot matches newline
    //
    #[case("abc", "a", ScopedView::new(RWScopes(vec![RWScope(In(B("a"))), RWScope(Out("bc"))])))]
    //
    #[case("abc", r"\w", ScopedView::new(RWScopes(vec![RWScope(In(B("a"))), RWScope(In(B("b"))), RWScope(In(B("c")))])))]
    #[case("abc", r"\W", ScopedView::new(RWScopes(vec![RWScope(Out("abc"))])))]
    #[case("abc", r"\w+", ScopedView::new(RWScopes(vec![RWScope(In(B("abc")))])))]
    //
    #[case("Work 69 on 420 words", r"\w+", ScopedView::new(RWScopes(vec![RWScope(In(B("Work"))), RWScope(Out(" ")), RWScope(In(B("69"))), RWScope(Out(" ")), RWScope(In(B("on"))), RWScope(Out(" ")), RWScope(In(B("420"))), RWScope(Out(" ")), RWScope(In(B("words")))])))]
    #[case("Ignore 69 the 420 digits", r"\p{letter}+", ScopedView::new(RWScopes(vec![RWScope(In(B("Ignore"))), RWScope(Out(" 69 ")), RWScope(In(B("the"))), RWScope(Out(" 420 ")), RWScope(In(B("digits")))])))]
    //
    #[case(".", ".", ScopedView::new(RWScopes(vec![RWScope(In(B(".")))])))]
    #[case(r"\.", ".", ScopedView::new(RWScopes(vec![RWScope(In(B(r"\"))), RWScope(In(B(".")))])))]
    #[case(r".", r"\.", ScopedView::new(RWScopes(vec![RWScope(In(B(r".")))])))]
    #[case(r"\.", r"\.", ScopedView::new(RWScopes(vec![RWScope(Out(r"\")), RWScope(In(B(r".")))])))]
    #[case(r"\w", r"\w", ScopedView::new(RWScopes(vec![RWScope(Out(r"\")), RWScope(In(B(r"w")))])))]
    //
    // Capture groups
    #[case(r"Hello", r"\w+", ScopedView::new(RWScopes(vec![RWScope(In(B(r"Hello")))])))]
    #[case(
        r"Hello", r"(\w+)",
        ScopedView::new(RWScopes(
            vec![
                RWScope(In(B(r"H"))),
                RWScope(In(B(r"e"))),
                RWScope(In(B(r"l"))),
                RWScope(In(B(r"l"))),
                RWScope(In(B(r"o")))
            ]
        ))
    )]
    #[case(
        r"Hello World", r"Hello (\w+)",
        ScopedView::new(RWScopes(
            vec![
                RWScope(In(B(r"Hello "))),
                RWScope(In(B(r"W"))),
                RWScope(In(B(r"o"))),
                RWScope(In(B(r"r"))),
                RWScope(In(B(r"l"))),
                RWScope(In(B(r"d")))
            ]
        ))
    )]
    #[case(
        // https://github.com/alexpovel/srgn/issues/71; this used to panic
        r#""error"; "x" => %x, "y" => %y"#,
        r"(?P<msg>.+);(?P<structure>.+)",
        ScopedView::new(RWScopes(
            vec![
                RWScope(In(B(r#"""#))),
                RWScope(In(B("e"))),
                RWScope(In(B("r"))),
                RWScope(In(B("r"))),
                RWScope(In(B("o"))),
                RWScope(In(B("r"))),
                RWScope(In(B(r#"""#))),
                RWScope(In(B(";"))),
                RWScope(In(B(" "))),
                RWScope(In(B(r#"""#))),
                RWScope(In(B("x"))),
                RWScope(In(B(r#"""#))),
                RWScope(In(B(" "))),
                RWScope(In(B("="))),
                RWScope(In(B(">"))),
                RWScope(In(B(" "))),
                RWScope(In(B("%"))),
                RWScope(In(B("x"))),
                RWScope(In(B(","))),
                RWScope(In(B(" "))),
                RWScope(In(B(r#"""#))),
                RWScope(In(B("y"))),
                RWScope(In(B(r#"""#))),
                RWScope(In(B(" "))),
                RWScope(In(B("="))),
                RWScope(In(B(">"))),
                RWScope(In(B(" "))),
                RWScope(In(B("%"))),
                RWScope(In(B("y"))),
            ]
        ))
    )]
    fn test_regex_scoping(
        #[case] input: &str,
        #[case] pattern: &str,
        #[case] expected: ScopedView,
    ) {
        let mut builder = crate::scoping::view::ScopedViewBuilder::new(input);
        let regex = Regex::new(RegexPattern::new(pattern).unwrap());
        builder.explode(&regex);
        let actual = builder.build();

        assert_eq!(actual, expected);
    }

    mod fuzzyish {
        use std::time::{Duration, Instant};

        use crate::scoping::scope::ROScope;

        use super::*;

        use log::info;
        use rand;
        use rand::seq::SliceRandom;
        use rand::Rng;
        use test_log::test;

        fn generate_random_regex(mut rng: &mut rand::rngs::ThreadRng) -> Option<RegexPattern> {
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

            RegexPattern::new(regex.as_str()).ok()
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
        fn test_regex_scoping_randomly() {
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

            loop {
                n_tries += 1;

                let Some(regex) = generate_random_regex(&mut rng) else {
                    continue;
                };
                let scope = Regex::new(regex);
                let input: String = rng.sample(&gen);

                let scopes = scope.scope(&input);

                if scopes.0.iter().any(|s| match s {
                    ROScope(In(_)) => true,
                    ROScope(Out(_)) => false,
                }) {
                    n_matches += 1;
                }

                let mut reassembled = String::new();
                for scope in scopes.0 {
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
                n_tries > 100,
                // Might happen in CI, but we should ensure a certain lower bound;
                // locally, many more tests can run.
                "Too few tries; is the host machine very slow?"
            );
        }
    }
}
