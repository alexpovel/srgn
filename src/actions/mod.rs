#[cfg(feature = "deletion")]
mod deletion;
#[cfg(feature = "german")]
mod german;
#[cfg(feature = "lower")]
mod lower;
#[cfg(feature = "normalization")]
mod normalization;
#[cfg(feature = "replace")]
mod replace;
#[cfg(feature = "symbols")]
mod symbols;
#[cfg(feature = "titlecase")]
mod titlecase;
#[cfg(feature = "upper")]
mod upper;

use std::fmt::Debug;

#[cfg(feature = "deletion")]
pub use deletion::Deletion;
#[cfg(feature = "german")]
pub use german::German;
#[cfg(feature = "lower")]
pub use lower::Lower;
#[cfg(feature = "normalization")]
pub use normalization::Normalization;
#[cfg(feature = "replace")]
pub use replace::Replacement;
#[cfg(feature = "symbols")]
pub use symbols::{inversion::SymbolsInversion, Symbols};
#[cfg(feature = "titlecase")]
pub use titlecase::Titlecase;
#[cfg(feature = "upper")]
pub use upper::Upper;

/// An action in the processing pipeline, as initiated by [`crate::apply`].
///
/// Actions are the core of the text processing pipeline and can be applied in any
/// order, [any number of times each](https://en.wikipedia.org/wiki/Idempotence) (more
/// than once being wasted work, though).
pub trait Action: Send + Sync + Debug {
    /// Apply this action to the given input.
    ///
    /// This is infallible: it cannot fail in the sense of [`Result`]. It can only
    /// return incorrect results, which would be bugs (please report).
    fn act(&self, input: &str) -> String;
}
