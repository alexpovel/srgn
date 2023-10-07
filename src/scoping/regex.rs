use super::{ScopedViewBuildStep, ScopedViewBuilder};
use crate::RegexPattern;
use crate::GLOBAL_SCOPE;
use log::{debug, trace};
use std::error::Error;
use std::fmt;
use std::ops::Range;

#[derive(Debug)]
pub struct Regex {
    pattern: RegexPattern,
}

impl Regex {
    #[must_use]
    pub fn new(pattern: RegexPattern) -> Self {
        Self { pattern }
    }
}

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

impl ScopedViewBuildStep for Regex {
    fn scope<'viewee>(&self, input: &'viewee str) -> ScopedViewBuilder<'viewee> {
        ScopedViewBuilder::new(input).explode_from_ranges(|s| {
            let has_capture_groups = self.pattern.captures_len() > 1;

            if !has_capture_groups {
                trace!(
                    "No capture groups in pattern '{}', short-circuiting",
                    s.escape_debug()
                );
                return self
                    .pattern
                    .find_iter(s)
                    .flatten()
                    .map(|m| m.range())
                    .collect();
            }

            trace!(
                "Pattern '{}' has capture groups, iterating over matches",
                s.escape_debug()
            );
            let mut ranges = Vec::new();
            for cap in self.pattern.captures_iter(s).flatten() {
                let mut it = cap.iter();

                let overall_match = it
                    .next()
                    // https://docs.rs/regex/1.9.5/regex/struct.SubCaptureMatches.html
                    .expect("Entered iterator of matches, but zeroth (whole) match missing")
                    .expect("First element guaranteed to be non-None (whole match)");
                trace!(
                    "Overall match: {:?} ({}..{})",
                    overall_match.as_str().escape_debug(),
                    overall_match.start(),
                    overall_match.end()
                );

                let mut subranges = Vec::new();
                for group in it.flatten() {
                    trace!(
                        "Group match: {:?} ({}..{})",
                        group.as_str().escape_debug(),
                        group.start(),
                        group.end()
                    );
                    subranges.push(group.range());
                }

                let mut last_end = overall_match.range().start;
                for subrange in subranges.into_iter().rev() {
                    ranges.push(Range {
                        start: last_end,
                        end: subrange.start,
                    });

                    ranges.extend(shatter(&subrange));

                    last_end = subrange.end;
                }
            }

            debug!("Ranges to scope after regex: {:?}", ranges);
            ranges
        })
    }
}

/// For a given [`Range`], shatters it into pieces of length 1, returning a [`Vec`] of
/// length equal to the length of the original range.
fn shatter(range: &Range<usize>) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();

    for i in range.start..range.end {
        ranges.push(Range {
            start: i,
            end: i + 1,
        });
    }

    ranges
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::scoping::{
        Scope::{In, Out},
        ScopedView,
    };
    use std::borrow::Cow::Borrowed as B;

    use super::*;

    #[rstest]
    #[case("a", "a", ScopedView::new(vec![In(B("a"))]))]
    #[case("aa", "a", ScopedView::new(vec![In(B("a")), In(B("a"))]))]
    #[case("aba", "a", ScopedView::new(vec![In(B("a")), Out("b"), In(B("a"))]))]
    //
    #[case("a", "", ScopedView::new(vec![Out("a")]))]
    #[case("", "a", ScopedView::new(vec![]))] // Empty results are discarded
    //
    #[case("a", "a", ScopedView::new(vec![In(B("a"))]))]
    #[case("a", "b", ScopedView::new(vec![Out("a")]))]
    //
    #[case("a", ".*", ScopedView::new(vec![In(B("a"))]))]
    #[case("a", ".+?", ScopedView::new(vec![In(B("a"))]))]
    //
    #[case("a\na", ".*", ScopedView::new(vec![In(B("a")), Out("\n"), In(B("a"))]))]
    #[case("a\na", "(?s).*", ScopedView::new(vec![In(B("a\na"))]))] // Dot matches newline
    //
    #[case("abc", "a", ScopedView::new(vec![In(B("a")), Out("bc")]))]
    //
    #[case("abc", r"\w", ScopedView::new(vec![In(B("a")), In(B("b")), In(B("c"))]))]
    #[case("abc", r"\W", ScopedView::new(vec![Out("abc")]))]
    #[case("abc", r"\w+", ScopedView::new(vec![In(B("abc"))]))]
    //
    #[case("Work 69 on 420 words", r"\w+", ScopedView::new(vec![In(B("Work")), Out(" "), In(B("69")), Out(" "), In(B("on")), Out(" "), In(B("420")), Out(" "), In(B("words"))]))]
    #[case("Ignore 69 the 420 digits", r"\p{letter}+", ScopedView::new(vec![In(B("Ignore")), Out(" 69 "), In(B("the")), Out(" 420 "), In(B("digits"))]))]
    //
    #[case(".", ".", ScopedView::new(vec![In(B("."))]))]
    #[case(r"\.", ".", ScopedView::new(vec![In(B(r"\")), In(B("."))]))]
    #[case(r".", r"\.", ScopedView::new(vec![In(B(r"."))]))]
    #[case(r"\.", r"\.", ScopedView::new(vec![Out(r"\"), In(B(r"."))]))]
    #[case(r"\w", r"\w", ScopedView::new(vec![Out(r"\"), In(B(r"w"))]))]
    //
    // Capture groups
    #[case(r"Hello", r"\w+", ScopedView::new(vec![In(B(r"Hello"))]))]
    #[case(
        r"Hello", r"(\w+)",
        ScopedView::new(
            vec![
                In(B(r"H")),
                In(B(r"e")),
                In(B(r"l")),
                In(B(r"l")),
                In(B(r"o"))
            ]
        )
    )]
    #[case(
        r"Hello World", r"Hello (\w+)",
        ScopedView::new(
            vec![
                In(B(r"Hello ")),
                In(B(r"W")),
                In(B(r"o")),
                In(B(r"r")),
                In(B(r"l")),
                In(B(r"d"))
            ]
        )
    )]
    fn test_regex_scoping(
        #[case] input: &str,
        #[case] pattern: &str,
        #[case] expected: ScopedView,
    ) {
        let regex = Regex::new(RegexPattern::new(pattern).unwrap());
        let actual = regex.scope(input).build();

        assert_eq!(actual, expected);
    }

    mod fuzzyish {
        use std::time::{Duration, Instant};

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

                let view = scope.scope(&input);

                if view.scopes.iter().any(|s| match s {
                    In(_) => true,
                    Out(_) => false,
                }) {
                    n_matches += 1;
                }

                let mut reassembled = String::new();
                for scope in view {
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
