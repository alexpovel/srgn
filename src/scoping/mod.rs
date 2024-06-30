//! Items for defining the scope actions are applied within.

use scope::RangesWithContext;

use crate::scoping::scope::ROScopes;
#[cfg(doc)]
use crate::scoping::{scope::Scope, view::ScopedView};

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
    fn scope<'viewee>(&self, input: &'viewee str) -> ROScopes<'viewee> {
        ROScopes::from_raw_ranges(input, self.scope_raw(input))
    }

    /// Scope the given `input`, returning raw ranges.
    ///
    /// Raw ranges are those not turned into [`ROScopes`] yet.
    fn scope_raw<'viewee>(&self, input: &'viewee str) -> RangesWithContext<'viewee>;
}

// https://www.reddit.com/r/rust/comments/droxdg/why_arent_traits_impld_for_boxdyn_trait/
impl Scoper for Box<dyn Scoper> {
    fn scope_raw<'viewee>(&self, input: &'viewee str) -> RangesWithContext<'viewee> {
        self.as_ref().scope_raw(input)
    }
}
