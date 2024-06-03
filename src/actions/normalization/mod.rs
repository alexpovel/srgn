use super::Action;
use unicode_categories::UnicodeCategories;
use unicode_normalization::UnicodeNormalization;

/// Performs Unicode normalization.
///
/// Uses NFD (Normalization Form D), canonical decomposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Normalization {}

impl Action for Normalization {
    fn act(&self, input: &str) -> String {
        input.nfd().filter(|c| !c.is_mark()).collect()
    }
}
