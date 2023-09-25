use super::ScopedViewBuildStep;
use std::fmt;
use tree_sitter::QueryError;
pub use tree_sitter::{Language, Parser, Query, QueryCursor};

pub mod python;

#[derive(Debug)]
pub enum LanguageScoperError {
    InvalidQuery(QueryError),
}

impl From<QueryError> for LanguageScoperError {
    fn from(e: QueryError) -> Self {
        Self::InvalidQuery(e)
    }
}

impl fmt::Display for LanguageScoperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidQuery(e) => write!(f, "Invalid query: {e}"),
        }
    }
}

pub trait LanguageScopedViewBuildStep: ScopedViewBuildStep {
    fn lang() -> Language;
    fn parser() -> Parser;
}
