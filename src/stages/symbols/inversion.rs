use crate::Stage;

use super::Symbol;

/// Inverts all symbols inserted by [`SymbolsStage`].
///
/// This is guaranteed to be the inverse of [`SymbolsStage`], as the replacements and
/// originals form a [bijection](https://en.wikipedia.org/wiki/Bijection).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct SymbolsInversionStage {}

impl Stage for SymbolsInversionStage {
    fn process(&self, input: &str) -> String {
        input
            .chars()
            .map(|c| match Symbol::try_from(c) {
                Ok(s) => match s {
                    // This is *horrible* as in the current implementation, we cannot
                    // access these symbols. They are implicitly encoded in the
                    // `substitute` method of `SymbolsStage`. As such, this inversion
                    // can get out of sync with the original. There is a property test
                    // in place to catch this.
                    Symbol::EmDash => "---",
                    Symbol::EnDash => "--",
                    Symbol::ShortRightArrow => "->",
                    Symbol::ShortLeftArrow => "<-",
                    Symbol::LongRightArrow => "-->",
                    Symbol::LongLeftArrow => "<--",
                    Symbol::LeftRightArrow => "<->",
                    Symbol::RightDoubleArrow => "=>",
                    Symbol::NotEqual => "!=",
                    Symbol::LessThanOrEqual => "<=",
                    Symbol::GreaterThanOrEqual => ">=",
                }
                .into(),
                Err(_) => c.to_string(),
            })
            .collect()
    }
}
