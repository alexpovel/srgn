use crate::scoping::scope::Scope::{In, Out};
use itertools::Itertools;
use log::{debug, trace};
use std::{borrow::Cow, ops::Range};

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

/// A read-only scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ROScope<'viewee>(pub Scope<'viewee, &'viewee str>);

/// Multiple read-only scopes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ROScopes<'viewee>(pub Vec<ROScope<'viewee>>);

/// A read-write scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RWScope<'viewee>(pub Scope<'viewee, Cow<'viewee, str>>);

/// Multiple read-write scopes.
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
    /// Construct a new instance from the given raw ranges.
    ///
    /// The passed `input` will be traversed according to `ranges`: all specified
    /// `ranges` are taken as [`In`] scope, everything not covered by a range is [`Out`]
    /// of scope.
    ///
    /// ## Panics
    ///
    /// Panics if the given `ranges` contain indices out-of-bounds for `input`.
    #[must_use]
    pub fn from_raw_ranges(input: &'viewee str, ranges: Vec<Range<usize>>) -> Self {
        let mut scopes = Vec::with_capacity(ranges.len());

        let mut last_end = 0;
        for Range { start, end } in ranges.into_iter().sorted_by_key(|r| r.start) {
            scopes.push(ROScope(Out(&input[last_end..start])));
            scopes.push(ROScope(In(&input[start..end])));
            last_end = end;
        }

        if last_end < input.len() {
            scopes.push(ROScope(Out(&input[last_end..])));
        }

        scopes.retain(|s| !s.is_empty());

        debug!("Scopes: {:?}", scopes);

        ROScopes(scopes)
    }

    /// Inverts the scopes: what was previously [`In`] is now [`Out`], and vice versa.
    #[must_use]
    pub fn invert(self) -> Self {
        trace!("Inverting scopes: {:?}", self.0);
        let scopes = self
            .0
            .into_iter()
            .map(|s| match s {
                ROScope(In(s)) => ROScope(Out(s)),
                ROScope(Out(s)) => ROScope(In(s)),
            })
            .collect();
        trace!("Inverted scopes: {:?}", scopes);

        Self(scopes)
    }
}

/// Checks for equality, regarding only raw [`str`] parts, i.e. disregards whether an
/// element is [`In`] or [`Out`] of scope.
impl PartialEq<&str> for ROScopes<'_> {
    fn eq(&self, other: &&str) -> bool {
        let mut start = 0;
        let mut end = None;

        for scope in &self.0 {
            let s: &str = scope.into();
            end = Some(start + s.len());

            let Some(substring) = other.get(start..end.unwrap()) else {
                return false;
            };

            if substring != s {
                return false;
            }

            start = end.unwrap();
        }

        match end {
            Some(e) => other.len() == e,
            None => other.is_empty(),
        }
    }
}

impl PartialEq<ROScopes<'_>> for &str {
    fn eq(&self, other: &ROScopes<'_>) -> bool {
        other == self
    }
}

impl<'viewee> From<&'viewee ROScope<'viewee>> for &'viewee str {
    /// Get the underlying string slice.
    ///
    /// All variants contain such a slice, so this is a convenient method.
    fn from(s: &'viewee ROScope) -> Self {
        match s.0 {
            In(s) | Out(s) => s,
        }
    }
}

impl<'viewee> From<ROScope<'viewee>> for RWScope<'viewee> {
    fn from(s: ROScope<'viewee>) -> Self {
        match s.0 {
            In(s) => RWScope(In(Cow::Borrowed(s))),
            Out(s) => RWScope(Out(s)),
        }
    }
}

impl<'viewee> From<&'viewee RWScope<'viewee>> for &'viewee str {
    /// Get the underlying string slice.
    ///
    /// All variants contain such a slice, so this is a convenient method.
    fn from(s: &'viewee RWScope) -> Self {
        match &s.0 {
            In(s) => s,
            Out(s) => s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    // Base cases
    #[case(ROScopes(vec![ROScope(In("abc"))]), "abc", true)]
    #[case(ROScopes(vec![ROScope(In("cba"))]), "cba", true)]
    #[case(ROScopes(vec![ROScope(In("🦀"))]), "🦀", true)]
    #[case(ROScopes(vec![ROScope(In("🦀"))]), "🤗", false)]
    //
    // Substring matching
    #[case(ROScopes(vec![ROScope(In("a")), ROScope(In("b"))]), "ab", true)]
    #[case(ROScopes(vec![ROScope(In("a")), ROScope(In("b")), ROScope(In("c"))]), "abc", true)]
    //
    #[case(ROScopes(vec![ROScope(In("a")), ROScope(In("b"))]), "ac", false)]
    #[case(ROScopes(vec![ROScope(In("a")), ROScope(In("b"))]), "a", false)]
    #[case(ROScopes(vec![ROScope(In("a")), ROScope(In("b"))]), "b", false)]
    #[case(ROScopes(vec![ROScope(In("a")), ROScope(In("b")), ROScope(In("c"))]), "acc", false)]
    //
    // Length mismatch
    #[case(ROScopes(vec![ROScope(In("abc"))]), "abcd", false)]
    #[case(ROScopes(vec![ROScope(In("abcd"))]), "abc", false)]
    //
    // Partial emptiness
    #[case(ROScopes(vec![ROScope(In("abc"))]), "", false)]
    #[case(ROScopes(vec![ROScope(In(""))]), "abc", false)]
    #[case(ROScopes(vec![ROScope(Out(""))]), "abc", false)]
    #[case(ROScopes(vec![ROScope(In("")), ROScope(Out(""))]), "abc", false)]
    //
    // Full emptiness
    #[case(ROScopes(vec![ROScope(In(""))]), "", true)]
    #[case(ROScopes(vec![ROScope(Out(""))]), "", true)]
    #[case(ROScopes(vec![ROScope(In("")), ROScope(Out(""))]), "", true)]
    //
    // Types of scope doesn't matter
    #[case(ROScopes(vec![ROScope(In("a"))]), "a", true)]
    #[case(ROScopes(vec![ROScope(Out("a"))]), "a", true)]
    fn test_scoped_view_str_equality(
        #[case] scopes: ROScopes<'_>,
        #[case] string: &str,
        #[case] equal: bool,
    ) {
        assert!((scopes == string) == equal);
    }
}
