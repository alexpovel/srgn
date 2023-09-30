use super::ScopedViewBuildStep;
use std::{error::Error, fmt};
use tree_sitter::QueryError;
pub use tree_sitter::{
    Language as TSLanguage, Parser as TSParser, Query as TSQuery, QueryCursor as TSQueryCursor,
};

pub mod python;

#[derive(Debug)]
pub enum LanguageScoperError {
    InvalidQuery(QueryError),
    NoSuchPremadeQuery(String),
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
            Self::NoSuchPremadeQuery(query) => write!(f, "No such premade query: {query}"),
        }
    }
}

impl Error for LanguageScoperError {}

pub trait LanguageScopedViewBuildStep: ScopedViewBuildStep {
    fn lang() -> TSLanguage;
    fn parser() -> TSParser;
}
