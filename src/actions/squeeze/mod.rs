use super::Action;
use crate::scoping::{Scope, ScopedView};
use log::{debug, trace};

/// Squeezes all consecutive matched scopes into a single occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Squeeze {}

impl Action for Squeeze {
    fn act(&self, _input: &str) -> String {
        unimplemented!("Action works without processing individual input")
    }

    fn map<'viewee, 'a>(&self, view: &'a mut ScopedView<'viewee>) -> &'a mut ScopedView<'viewee> {
        debug!("Squeezing input by collapsing all consecutive in-scope occurrences.");
        let v = view.into_inner_mut();

        let mut prev_was_in = false;
        v.retain(|scope| {
            let keep = !(prev_was_in && matches!(scope, Scope::In(_)));
            prev_was_in = matches!(scope, Scope::In(_));
            trace!("keep: {}, scope: {:?}", keep, scope);
            keep
        });

        debug!("Squeezed: {:?}", v);

        view
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoping::ScopedViewBuilder;
    use crate::RegexPattern;
    use rstest::rstest;

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
    // Squeezes only the pattern, no other repetitions
    #[case("aaabbb", "a", "abbb")]
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
    // Deals with overlapping matches; behavior of `regex` crate
    #[case("abab", r"aba", "abab")]
    #[case("ababa", r"aba", "ababa")]
    #[case("ababab", r"aba", "ababab")]
    #[case("abababa", r"aba", "abababa")]
    #[case("aba", r"aba", "aba")]
    #[case("abaaba", r"aba", "aba")]
    //
    // Requires non-greedy matches for meaningful results
    #[case("ab", r"\s+?", "ab")]
    #[case("a b", r"\s+?", "a b")]
    #[case("a\t\tb", r"\s+?", "a\tb")]
    #[case("a\t\t  b", r"\s+?", "a\tb")]
    //
    // Deals with more complex patterns
    #[case("ab", "", "ab")] // Matches nothing
    //
    #[case("ab", r"[ab]", "a")]
    #[case("ab", r"[ab]+", "ab")]
    #[case("ab", r"[ab]+?", "a")]
    //
    #[case("abab", r"\D", "a")]
    //
    // Builds up properly; need non-capturing group
    #[case("abab", r"(?:ab){2}", "abab")]
    #[case("ababa", r"(?:ab){2}", "ababa")]
    #[case("ababab", r"(?:ab){2}", "ababab")]
    #[case("abababa", r"(?:ab){2}", "abababa")]
    #[case("abababab", r"(?:ab){2}", "abab")]
    #[case("ababababa", r"(?:ab){2}", "ababa")]
    #[case("ababababab", r"(?:ab){2}", "ababab")]
    #[case("abababababab", r"(?:ab){2}", "abab")]
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
    fn test_squeeze(#[case] input: &str, #[case] pattern: RegexPattern, #[case] expected: &str) {
        let action = Squeeze {};

        let builder = ScopedViewBuilder::new(input)
            .explode_from_scoper(&crate::scoping::regex::Regex::new(pattern.clone()));
        let mut view = builder.build();

        action.map(&mut view);
        let result = view.to_string();

        assert_eq!(result, expected);
    }
}
