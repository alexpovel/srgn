use itertools::Itertools;
use log::debug;
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
