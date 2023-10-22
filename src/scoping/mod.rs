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

pub trait ScopedViewBuildStep {
    fn scope<'viewee>(&self, input: &'viewee str) -> ScopedViewBuilder<'viewee>;
}

impl fmt::Debug for dyn ScopedViewBuildStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scoper").finish()
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

type ROScope<'viewee> = Scope<'viewee, &'viewee str>;
type ROScopes<'viewee> = Vec<ROScope<'viewee>>;

type RWScope<'viewee> = Scope<'viewee, Cow<'viewee, str>>;
type RWScopes<'viewee> = Vec<RWScope<'viewee>>;

impl<'viewee> ROScope<'viewee> {
    /// Check whether the scope is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let s: &str = self.into();
        s.is_empty()
    }
}

impl<'viewee> From<&'viewee ROScope<'viewee>> for &'viewee str {
    /// Get the underlying string slice of a [`ScopeStatus`].
    ///
    /// All variants contain such a slice, so this is a convenient method.
    fn from(s: &'viewee ROScope) -> Self {
        match s {
            Scope::In(s) | Scope::Out(s) => s,
        }
    }
}

impl<'viewee> From<ROScope<'viewee>> for RWScope<'viewee> {
    fn from(s: ROScope<'viewee>) -> Self {
        match s {
            Scope::In(s) => RWScope::In(Cow::Borrowed(s)),
            Scope::Out(s) => RWScope::Out(s),
        }
    }
}

impl<'viewee> From<&'viewee RWScope<'viewee>> for &'viewee str {
    /// Get the underlying string slice of a [`ScopeStatus`].
    ///
    /// All variants contain such a slice, so this is a convenient method.
    fn from(s: &'viewee RWScope) -> Self {
        match s {
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
            scopes: vec![Scope::In(input)],
        }
    }

    #[must_use]
    pub fn build(self) -> ScopedView<'viewee> {
        ScopedView {
            scopes: self
                .scopes
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
        }
    }
}

impl<'viewee> IntoIterator for ScopedViewBuilder<'viewee> {
    type Item = ROScope<'viewee>;

    type IntoIter = std::vec::IntoIter<ROScope<'viewee>>;

    fn into_iter(self) -> Self::IntoIter {
        self.scopes.into_iter()
    }
}

impl<'viewee> ScopedViewBuilder<'viewee> {
    #[must_use]
    pub fn explode_from_ranges(self, exploder: impl Fn(&str) -> Vec<Range<usize>>) -> Self {
        self.explode(|s| {
            trace!("Exploding from ranges: {:?}", s);

            let ranges = exploder(s);
            trace!("Raw ranges after exploding: {:?}", ranges);

            let mut scopes = Vec::new();

            let mut last_end = 0;
            for Range { start, end } in ranges.into_iter().sorted_by_key(|r| r.start) {
                scopes.push(Scope::Out(&s[last_end..start]));
                scopes.push(Scope::In(&s[start..end]));
                last_end = end;
            }

            if last_end < s.len() {
                scopes.push(Scope::Out(&s[last_end..]));
            }

            scopes.retain(|s| !s.is_empty());

            debug!("Scopes: {:?}", scopes);

            ScopedViewBuilder { scopes }
        })
    }

    #[must_use]
    pub fn explode_from_scoper(self, scoper: &impl ScopedViewBuildStep) -> Self {
        self.explode(|s| scoper.scope(s))
    }

    #[must_use]
    pub fn explode<F>(mut self, exploder: F) -> Self
    where
        F: Fn(&'viewee str) -> Self,
    {
        trace!("Exploding scopes: {:?}", self.scopes);
        let mut new = Vec::with_capacity(self.scopes.len());
        for scope in self.scopes.drain(..) {
            trace!("Exploding scope: {:?}", scope);

            if scope.is_empty() {
                trace!("Skipping empty scope");
                continue;
            }

            match scope {
                Scope::In(s) => {
                    let mut new_scopes = exploder(s).scopes;
                    new_scopes.retain(|s| !s.is_empty());
                    new.extend(new_scopes);
                }
                // Be explicit about the `Out(_)` case, so changing the enum is a
                // compile error
                Scope::Out("") => {}
                out @ Scope::Out(_) => new.push(out),
            }

            trace!("Exploded scope, new scopes are: {:?}", new);
        }
        trace!("Done exploding scopes.");

        ScopedViewBuilder { scopes: new }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedView<'viewee> {
    scopes: RWScopes<'viewee>,
}

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

    /// submit a function to be applied to each in-scope, returning out-scopes unchanged
    pub fn map<F>(&mut self, f: &F) -> &mut Self
    where
        F: Fn(&str) -> <str as ToOwned>::Owned,
    {
        for scope in &mut self.scopes {
            match scope {
                Scope::In(s) => {
                    let res = f(s);
                    debug!(
                        "Replacing '{}' with '{}'",
                        s.escape_debug(),
                        res.escape_debug()
                    );
                    *scope = Scope::In(Cow::Owned(res));
                }
                Scope::Out(s) => {
                    debug!("Appending '{}'", s.escape_debug());
                }
            }
        }

        self
    }

    pub fn into_inner_mut(&mut self) -> &mut RWScopes<'viewee> {
        self.scopes.as_mut()
    }

    /// Check whether anything is in scope.
    #[must_use]
    pub fn has_any_in_scope(&self) -> bool {
        self.scopes.iter().any(|s| match s {
            Scope::In(_) => true,
            Scope::Out(_) => false,
        })
    }
}

/// Implementations of all available actions as dedicated methods.
///
/// Where actions don't take arguments, neither do the methods.
impl<'viewee> ScopedView<'viewee> {
    pub fn map_action<A: Action>(&mut self, action: &A) -> &mut Self {
        self.map(&|s| action.act(s))
    }

    pub fn delete(&mut self) -> &mut Self {
        let action = actions::Deletion::default();

        self.map_action(&action)
    }

    pub fn german(&mut self) -> &mut Self {
        let action = actions::German::default();

        self.map_action(&action)
    }

    pub fn lower(&mut self) -> &mut Self {
        let action = actions::Lower::default();

        self.map_action(&action)
    }

    pub fn normalize(&mut self) -> &mut Self {
        let action = actions::Normalization::default();

        self.map_action(&action)
    }

    pub fn replace(&mut self, replacement: String) -> &mut Self {
        let action = actions::Replacement::new(replacement);

        self.map_action(&action)
    }

    pub fn squeeze(&mut self) -> &mut Self {
        let action = actions::Squeeze::default();

        self.map_action(&action)
    }

    pub fn symbols(&mut self) -> &mut Self {
        let action = actions::Symbols::default();

        self.map_action(&action)
    }

    pub fn titlecase(&mut self) -> &mut Self {
        let action = actions::Titlecase::default();

        self.map_action(&action)
    }

    pub fn upper(&mut self) -> &mut Self {
        let action = actions::Upper::default();

        self.map_action(&action)
    }
}

impl fmt::Display for ScopedView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for scope in &self.scopes {
            let s: &str = scope.into();
            write!(f, "{s}")?;
        }
        Ok(())
    }
}
