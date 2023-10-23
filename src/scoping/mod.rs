//! Items for defining the scope actions are applied within.

use crate::actions::{self, Action};

use self::literal::LiteralError;
use self::regex::RegexError;
use itertools::Itertools;
use log::{debug, trace};
use std::fmt;
use std::{borrow::Cow, ops::Range};

pub mod langs;
pub mod literal;
pub mod regex;

#[derive(Debug)]
pub enum ScoperBuildError {
    EmptyScope,
    RegexError(RegexError),
    LiteralError(LiteralError),
}

impl From<LiteralError> for ScoperBuildError {
    fn from(e: LiteralError) -> Self {
        Self::LiteralError(e)
    }
}

impl From<RegexError> for ScoperBuildError {
    fn from(e: RegexError) -> Self {
        Self::RegexError(e)
    }
}

/// Indicates whether a given string part is in scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope<'viewee, T> {
    /// The given part is in scope for processing.
    In(T),
    /// The given part is out of scope for processing.
    ///
    /// Treated as immutable, view-only.
    Out(&'viewee str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ROScope<'viewee>(pub Scope<'viewee, &'viewee str>);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ROScopes<'viewee>(pub Vec<ROScope<'viewee>>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RWScope<'viewee>(pub Scope<'viewee, Cow<'viewee, str>>);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RWScopes<'viewee>(pub Vec<RWScope<'viewee>>);

impl<'viewee> ROScope<'viewee> {
    /// Check whether the scope is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let s: &str = self.into();
        s.is_empty()
    }
}

impl<'viewee> ROScopes<'viewee> {
    #[must_use]
    pub fn from_raw_ranges(input: &'viewee str, ranges: Vec<Range<usize>>) -> Self {
        let mut scopes = Vec::with_capacity(ranges.len());

        let mut last_end = 0;
        for Range { start, end } in ranges.into_iter().sorted_by_key(|r| r.start) {
            scopes.push(ROScope(Scope::Out(&input[last_end..start])));
            scopes.push(ROScope(Scope::In(&input[start..end])));
            last_end = end;
        }

        if last_end < input.len() {
            scopes.push(ROScope(Scope::Out(&input[last_end..])));
        }

        scopes.retain(|s| !s.is_empty());

        debug!("Scopes: {:?}", scopes);

        ROScopes(scopes)
    }
}

pub trait Scoper {
    fn scope<'viewee>(&self, input: &'viewee str) -> ROScopes<'viewee>;
}

impl fmt::Debug for dyn Scoper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scoper").finish()
    }
}

impl<'viewee> From<&'viewee ROScope<'viewee>> for &'viewee str {
    /// Get the underlying string slice.
    ///
    /// All variants contain such a slice, so this is a convenient method.
    fn from(s: &'viewee ROScope) -> Self {
        match s.0 {
            Scope::In(s) | Scope::Out(s) => s,
        }
    }
}

impl<'viewee> From<ROScope<'viewee>> for RWScope<'viewee> {
    fn from(s: ROScope<'viewee>) -> Self {
        match s.0 {
            Scope::In(s) => RWScope(Scope::In(Cow::Borrowed(s))),
            Scope::Out(s) => RWScope(Scope::Out(s)),
        }
    }
}

impl<'viewee> From<&'viewee RWScope<'viewee>> for &'viewee str {
    /// Get the underlying string slice.
    ///
    /// All variants contain such a slice, so this is a convenient method.
    fn from(s: &'viewee RWScope) -> Self {
        match &s.0 {
            Scope::In(s) => s,
            Scope::Out(s) => s,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedViewBuilder<'viewee> {
    scopes: ROScopes<'viewee>,
}

impl<'viewee> ScopedViewBuilder<'viewee> {
    #[must_use]
    pub fn new(input: &'viewee str) -> Self {
        Self {
            scopes: ROScopes(vec![ROScope(Scope::In(input))]),
        }
    }

    #[must_use]
    pub fn build(self) -> ScopedView<'viewee> {
        ScopedView {
            scopes: RWScopes(
                self.scopes
                    .0
                    .into_iter()
                    .map(std::convert::Into::into)
                    .collect(),
            ),
        }
    }
}

impl<'viewee> IntoIterator for ScopedViewBuilder<'viewee> {
    type Item = ROScope<'viewee>;

    type IntoIter = std::vec::IntoIter<ROScope<'viewee>>;

    fn into_iter(self) -> Self::IntoIter {
        self.scopes.0.into_iter()
    }
}

impl<'viewee> ScopedViewBuilder<'viewee> {
    #[must_use]
    pub fn explode_from_scoper(self, scoper: &impl Scoper) -> Self {
        self.explode(|s| scoper.scope(s))
    }

    #[must_use]
    pub fn explode<F>(mut self, exploder: F) -> Self
    where
        F: Fn(&'viewee str) -> ROScopes<'viewee>,
    {
        trace!("Exploding scopes: {:?}", self.scopes);
        let mut new = Vec::with_capacity(self.scopes.0.len());
        for scope in self.scopes.0.drain(..) {
            trace!("Exploding scope: {:?}", scope);

            if scope.is_empty() {
                trace!("Skipping empty scope");
                continue;
            }

            match scope {
                ROScope(Scope::In(s)) => {
                    let mut new_scopes = exploder(s);
                    new_scopes.0.retain(|s| !s.is_empty());
                    new.extend(new_scopes.0);
                }
                // Be explicit about the `Out(_)` case, so changing the enum is a
                // compile error
                ROScope(Scope::Out("")) => {}
                out @ ROScope(Scope::Out(_)) => new.push(out),
            }

            trace!("Exploded scope, new scopes are: {:?}", new);
        }
        trace!("Done exploding scopes.");

        ScopedViewBuilder {
            scopes: ROScopes(new),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedView<'viewee> {
    scopes: RWScopes<'viewee>,
}

/// Core implementations.
impl<'viewee> ScopedView<'viewee> {
    #[must_use]
    pub fn new(scopes: RWScopes<'viewee>) -> Self {
        Self { scopes }
    }

    /// For API discoverability.
    #[must_use]
    pub fn builder(input: &'viewee str) -> ScopedViewBuilder<'viewee> {
        ScopedViewBuilder::new(input)
    }

    /// Apply an action to all in-scope occurrences.
    ///
    /// See implementors of [`Action`] for available types.
    pub fn map(&mut self, action: &impl Action) -> &mut Self {
        for scope in &mut self.scopes.0 {
            match scope {
                RWScope(Scope::In(s)) => {
                    let res = action.act(s);
                    debug!(
                        "Replacing '{}' with '{}'",
                        s.escape_debug(),
                        res.escape_debug()
                    );
                    *scope = RWScope(Scope::In(Cow::Owned(res)));
                }
                RWScope(Scope::Out(s)) => {
                    debug!("Appending '{}'", s.escape_debug());
                }
            }
        }

        self
    }

    /// Squeeze all consecutive [`Scope::In`] scopes into a single occurrence (the first
    /// one).
    pub fn squeeze(&mut self) -> &mut Self {
        debug!("Squeezing view by collapsing all consecutive in-scope occurrences.");

        let mut prev_was_in = false;
        self.scopes.0.retain(|scope| {
            let keep = !(prev_was_in && matches!(scope, RWScope(Scope::In(_))));
            prev_was_in = matches!(scope, RWScope(Scope::In(_)));
            trace!("keep: {}, scope: {:?}", keep, scope);
            keep
        });

        debug!("Squeezed: {:?}", self.scopes);

        self
    }

    /// Check whether anything is in scope.
    #[must_use]
    pub fn has_any_in_scope(&self) -> bool {
        self.scopes.0.iter().any(|s| match s {
            RWScope(Scope::In(_)) => true,
            RWScope(Scope::Out(_)) => false,
        })
    }
}

/// Implementations of all available actions as dedicated methods.
///
/// Where actions don't take arguments, neither do the methods.
impl<'viewee> ScopedView<'viewee> {
    pub fn delete(&mut self) -> &mut Self {
        let action = actions::Deletion::default();

        self.map(&action)
    }

    pub fn german(&mut self) -> &mut Self {
        let action = actions::German::default();

        self.map(&action)
    }

    pub fn lower(&mut self) -> &mut Self {
        let action = actions::Lower::default();

        self.map(&action)
    }

    pub fn normalize(&mut self) -> &mut Self {
        let action = actions::Normalization::default();

        self.map(&action)
    }

    pub fn replace(&mut self, replacement: String) -> &mut Self {
        let action = actions::Replacement::new(replacement);

        self.map(&action)
    }

    pub fn symbols(&mut self) -> &mut Self {
        let action = actions::Symbols::default();

        self.map(&action)
    }

    pub fn invert_symbols(&mut self) -> &mut Self {
        let action = actions::SymbolsInversion::default();

        self.map(&action)
    }

    pub fn titlecase(&mut self) -> &mut Self {
        let action = actions::Titlecase::default();

        self.map(&action)
    }

    pub fn upper(&mut self) -> &mut Self {
        let action = actions::Upper::default();

        self.map(&action)
    }
}

impl fmt::Display for ScopedView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for scope in &self.scopes.0 {
            let s: &str = scope.into();
            write!(f, "{s}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
        let builder = ScopedViewBuilder::new(input)
            .explode_from_scoper(&crate::scoping::regex::Regex::new(pattern.clone()));
        let mut view = builder.build();

        view.squeeze();
        let result = view.to_string();

        assert_eq!(result, expected);
    }
}
