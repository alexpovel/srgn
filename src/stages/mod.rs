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
#[cfg(feature = "squeeze")]
mod squeeze;
#[cfg(feature = "symbols")]
mod symbols;
#[cfg(feature = "titlecase")]
mod titlecase;
#[cfg(feature = "upper")]
mod upper;

use std::fmt::Debug;

#[cfg(feature = "deletion")]
pub use deletion::DeletionStage;
#[cfg(feature = "german")]
pub use german::GermanStage;
#[cfg(feature = "lower")]
pub use lower::LowerStage;
#[cfg(feature = "normalization")]
pub use normalization::NormalizationStage;
#[cfg(feature = "replace")]
pub use replace::ReplacementStage;
#[cfg(feature = "squeeze")]
pub use squeeze::SqueezeStage;
#[cfg(feature = "symbols")]
pub use symbols::{inversion::SymbolsInversionStage, SymbolsStage};
#[cfg(feature = "titlecase")]
pub use titlecase::TitlecaseStage;
#[cfg(feature = "upper")]
pub use upper::UpperStage;

use crate::scoping::ScopedView;

/// A stage in the processing pipeline, as initiated by [`crate::apply`].
///
/// Stages are the core of the text processing pipeline and can be applied in any order,
/// [any number of times each](https://en.wikipedia.org/wiki/Idempotence) (more than
/// once being wasted work, though).
pub trait Stage: Send + Sync + Debug {
    /// Apply this stage to the given [`ScopedView`].
    ///
    /// This is infallible: it cannot fail in the sense of [`Result`]. It can only
    /// return incorrect results, which would be bugs (please report).
    fn process(&self, input: &str) -> String;

    /// Applies this stage to an `input`, working only on [`InScope`] items and
    /// forwarding [`OutOfScope`] items unchanged.
    ///
    /// Always returns an owned version of the `input`, even for stages where that might
    /// technically be unnecessary.
    ///
    /// This is infallible: it cannot fail in the sense of [`Result`]. It can only
    /// return incorrect results, which would be bugs (please report).
    fn map<'viewee, 'a>(&self, view: &'a mut ScopedView<'viewee>) -> &'a mut ScopedView<'viewee> {
        view.map(&|s| self.process(s))
    }
}
