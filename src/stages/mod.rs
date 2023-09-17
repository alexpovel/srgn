#[cfg(feature = "deletion")]
mod deletion;
#[cfg(feature = "german")]
mod german;
#[cfg(feature = "lower")]
mod lower;
#[cfg(feature = "replace")]
mod replace;
#[cfg(feature = "squeeze")]
mod squeeze;
#[cfg(feature = "symbols")]
mod symbols;
#[cfg(feature = "upper")]
mod upper;

use std::fmt::Debug;

pub use deletion::DeletionStage;
pub use german::GermanStage;
pub use lower::LowerStage;
pub use replace::ReplacementStage;
pub use squeeze::SqueezeStage;
pub use symbols::inversion::SymbolsInversionStage;
pub use symbols::SymbolsStage;
pub use upper::UpperStage;

use crate::scoped::{
    Scope,
    ScopeStatus::{InScope, OutOfScope},
    Scoped,
};

/// A stage in the processing pipeline, as initiated by [`crate::apply`].
///
/// Stages are the core of the text processing pipeline and can be applied in any order,
/// [any number of times each](https://en.wikipedia.org/wiki/Idempotence) (more than
/// once being wasted work, though).
pub trait Stage: Send + Sync + Scoped + Debug {
    /// Substitute text in a given `input` string.
    ///
    /// This is infallible: it cannot fail in the sense of [`Result`]. It can only
    /// return incorrect results, which would be bugs (please report).
    fn substitute(&self, input: &str) -> String;

    /// Applies this stage to an `input`, working only on [`InScope`] items and
    /// forwarding [`OutOfScope`] items unchanged.
    ///
    /// Always returns an owned version of the `input`, even for stages where that might
    /// technically be unnecessary.
    ///
    /// This is infallible: it cannot fail in the sense of [`Result`]. It can only
    /// return incorrect results, which would be bugs (please report).
    fn apply(&self, input: &str, scope: &Scope) -> String {
        let mut out = String::with_capacity(input.len());

        for scope in self.split_by_scope(input, scope) {
            match scope {
                InScope(s) => out.push_str(&self.substitute(s)),
                OutOfScope(s) => out.push_str(s),
            }
        }

        out
    }
}
