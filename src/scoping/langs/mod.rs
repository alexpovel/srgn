pub use tree_sitter::{Language, Parser, Query, QueryCursor};

use super::ScopedViewBuildStep;

pub mod python;

pub trait LanguageScopedViewBuildStep: ScopedViewBuildStep {
    fn lang() -> Language;
    fn parser() -> Parser;
}
