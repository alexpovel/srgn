//! Items for defining the scope actions are applied within.

use crate::scoping::scope::ROScopes;
#[cfg(doc)]
use crate::scoping::{scope::Scope, view::ScopedView};
use std::fmt;

/// Fixes for DOS-style line endings.
pub mod dosfix;
/// Create scoped views using programming language grammar-aware types.
pub mod langs;
/// Create scoped views using string literals.
pub mod literal;
/// Create scoped views using regular expressions.
pub mod regex;
/// [`Scope`] and its various wrappers.
pub mod scope;
/// [`ScopedView`] and its related types.
pub mod view;

/// An item capable of scoping down a given input into individual scopes.
pub trait Scoper: Send + Sync {
    /// Scope the given `input`.
    ///
    /// After application, the returned scopes are a collection of either in-scope or
    /// out-of-scope parts of the input. Assembling them back together should yield the
    /// original input.
    fn scope<'viewee>(&self, input: &'viewee str) -> ROScopes<'viewee>;
}

impl fmt::Debug for dyn Scoper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scoper").finish()
    }
}

impl<T> Scoper for T
where
    T: Fn(&str) -> ROScopes + Send + Sync,
{
    fn scope<'viewee>(&self, input: &'viewee str) -> ROScopes<'viewee> {
        self(input)
    }
}

// https://www.reddit.com/r/rust/comments/droxdg/why_arent_traits_impld_for_boxdyn_trait/
impl Scoper for Box<dyn Scoper> {
    fn scope<'viewee>(&self, input: &'viewee str) -> ROScopes<'viewee> {
        self.as_ref().scope(input)
    }
}
