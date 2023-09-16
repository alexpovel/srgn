use std::ops::Range;

use super::{tooling::StageResult, Stage};
use regex::Regex;

/// Deletes all matches of a given regex.
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct SqueezeStage {
    pattern: Regex,
}

impl Stage for SqueezeStage {
    fn substitute(&self, input: &str) -> StageResult {
        // Wouldn't need an owned `String` for this stage, but return signature requires
        // it anyway.
        let mut out = String::with_capacity(input.len());

        let mut left = 0; // Left bound of current substring we *might* push
        let mut previous: Option<regex::Match> = None;

        for m in self.pattern.find_iter(input) {
            let flush = previous.map_or(true, |p| !ranges_are_consecutive(&p.range(), &m.range()));

            if flush {
                out.push_str(&input[left..m.end()]);
            }

            left = m.end();
            previous = Some(m);
        }

        out.push_str(&input[left..]); // Remainder; entire string if no matches

        Ok(out.into())
    }
}

fn ranges_are_consecutive<T: Eq>(left: &Range<T>, right: &Range<T>) -> bool {
    left.end == right.start
}

impl SqueezeStage {
    /// Create a new instance.
    ///
    /// # Arguments
    ///
    /// * `pattern`: The regex to use for squeezing.
    ///
    /// # Panics
    ///
    /// Panics if the given pattern cannot be prepended with `(?U)`, which is used to
    /// [render greedy quantifiers
    /// non-greedy](https://docs.rs/regex/latest/regex/#grouping-and-flags), and vice
    /// versa.
    #[must_use]
    pub fn new(pattern: &Regex) -> Self {
        let pattern = Regex::new(&format!(r"(?U){pattern}"))
            .expect("should be able to prepend (?U) to pattern");

        Self { pattern }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    // Pattern only
    #[case("a", "a", "a")]
    #[case("aa", "a", "a")]
    #[case("aaa", "a", "a")]
    //
    // Pattern once; nothing to squeeze
    #[case("aba", "a", "aba")]
    #[case("bab", "a", "bab")]
    #[case("babab", "a", "babab")]
    #[case("ababa", "a", "ababa")]
    //
    // Squeezes start
    #[case("aab", "a", "ab")]
    //
    // Squeezes middle
    #[case("baab", "a", "bab")]
    //
    // Squeezes end
    #[case("abaa", "a", "aba")]
    //
    // Squeezes as soon as pattern occurs at least twice
    #[case("a", "ab", "a")]
    #[case("ab", "ab", "ab")]
    #[case("aba", "ab", "aba")]
    #[case("abab", "ab", "ab")]
    #[case("ababa", "ab", "aba")]
    #[case("ababab", "ab", "ab")]
    //
    // Squeezes nothing if pattern not present
    #[case("", "b", "")]
    #[case("a", "b", "a")]
    #[case("aa", "b", "aa")]
    #[case("aaa", "b", "aaa")]
    //
    // Deals with character classes (space)
    #[case("Hello World", r"\s", "Hello World")]
    #[case("Hello  World", r"\s", "Hello World")]
    #[case("Hello       World", r"\s", "Hello World")]
    #[case("Hello\tWorld", r"\t", "Hello\tWorld")]
    #[case("Hello\t\tWorld", r"\t", "Hello\tWorld")]
    //
    // Deals with character classes (inverted space)
    #[case("Hello World", r"\S", "H W")]
    #[case("Hello\t\tWorld", r"\S", "H\t\tW")]
    //
    // Turns greedy quantifiers into non-greedy ones automatically
    #[case("ab", r"\s+", "ab")]
    #[case("a b", r"\s+", "a b")]
    #[case("a\t\tb", r"\s+", "a\tb")]
    //
    // Turns greedy quantifiers into non-greedy ones automatically, even if user
    // specified themselves (extra option ignored)
    #[case("ab", r"(?U)\s+", "ab")]
    #[case("a b", r"(?U)\s+", "a b")]
    #[case("a\t\tb", r"(?U)\s+", "a\tb")]
    //
    // Inversion works (if user specified non-greedy manually, it becomes greedy). `+`
    // and `*` will make the concept of 'consecutive matches' meaningless!
    #[case("ab", r"\s+?", "ab")]
    #[case("a b", r"\s+?", "a b")]
    #[case("a\t\tb", r"\s+?", "a\t\tb")]
    #[case("a\t\t\t\tb", r"\s+?", "a\t\t\t\tb")]
    //
    // Deals with more complex patterns
    #[case("ab", "", "ab")] // Matches nothing
    //
    #[case("ab", r"[ab]", "a")]
    #[case("ab", r"[ab]+", "a")]
    #[case("ab", r"[ab]+?", "ab")]
    //
    #[case("abab", r"\D", "a")]
    //
    #[case("abab", r"(ab){2}", "abab")]
    #[case("ababa", r"(ab){2}", "ababa")]
    #[case("ababab", r"(ab){2}", "ababab")]
    #[case("abababa", r"(ab){2}", "abababa")]
    #[case("abababab", r"(ab){2}", "abab")]
    #[case("ababababa", r"(ab){2}", "ababa")]
    #[case("ababababab", r"(ab){2}", "ababab")]
    #[case("abababababab", r"(ab){2}", "abab")]
    //
    #[case("Anything whatsoever gets rEkT", r".", "A")]
    #[case(
        "Anything whatsoever gets rEkT",
        r".*", // Greediness inverted
        "Anything whatsoever gets rEkT"
    )]
    //
    // Deals with Unicode shenanigans
    #[case("😎😎", r"😎", "😎")]
    #[case("\0😎\0😎\0", r"😎", "\0😎\0😎\0")]
    //
    #[case("你你好", r"你", "你好")]
    //
    // Longer ("integration") tests; things that come up in the wild
    #[case(
        " dirty Strings  \t with  \t\t messed up  whitespace\n\n\n",
        r"\s",
        " dirty Strings with messed up whitespace\n"
    )]
    #[case(
        " dirty Strings  \t with  \t\t messed up  whitespace\n\n\n",
        r" ",
        " dirty Strings \t with \t\t messed up whitespace\n\n\n"
    )]
    fn test_squeeze(#[case] input: &str, #[case] pattern: Regex, #[case] expected: &str) {
        let stage = SqueezeStage::new(&pattern);

        let result: String = stage.substitute(input).unwrap().into();

        assert_eq!(result, expected);
    }
}
