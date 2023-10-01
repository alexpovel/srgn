use super::{ScopedViewBuildStep, ScopedViewBuilder};
use std::{error::Error, fmt};
use tree_sitter::QueryError;
pub use tree_sitter::{
    Language as TSLanguage, Parser as TSParser, Query as TSQuery, QueryCursor as TSQueryCursor,
};

pub mod python;

#[derive(Debug)]
pub struct Language<Q> {
    query: Q,
}

impl<Q> Language<Q> {
    pub fn new(query: Q) -> Self {
        Self { query }
    }

    // pub fn from_premade<P>(query: &P) -> Self
    // where
    //     for<'p> &'p P: Into<TSQuery>,
    // {
    //     Self::new(CodeQuery::Premade(query))
    // }
}

#[derive(Debug, Clone)]
pub enum CodeQuery<C, P>
where
    C: TryFrom<String> + Into<TSQuery>,
    // for<'c> &'c C: Into<TSQuery>,
    // for<'p> &'p P: Into<TSQuery>,
    P: Into<TSQuery>,
{
    Custom(C),
    Premade(P),
}

impl<C, P> From<CodeQuery<C, P>> for TSQuery
where
    C: TryFrom<String> + Into<TSQuery>,
    // for<'c> &'c C: Into<TSQuery>,
    // for<'p> &'p P: Into<TSQuery>,
    P: Into<TSQuery>,
{
    fn from(value: CodeQuery<C, P>) -> Self {
        match value {
            CodeQuery::Custom(query) => query.into(),
            CodeQuery::Premade(query) => query.into(),
        }
    }
}

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
    fn query(&self) -> TSQuery;

    #[must_use]
    fn parser() -> TSParser {
        let mut parser = TSParser::new();
        parser
            .set_language(Self::lang())
            .expect("Should be able to load language grammar and parser");

        parser
    }

    fn scope_via_query<'a>(&self, input: &'a str) -> ScopedViewBuilder<'a> {
        ScopedViewBuilder::new(input).explode_from_ranges(|s| {
            // tree-sitter is about incremental parsing, which we don't use here
            let old_tree = None;

            let tree = Self::parser()
                .parse(s, old_tree)
                .expect("No language set in parser, or other unrecoverable error");
            let root = tree.root_node();

            let mut qc = TSQueryCursor::new();
            let query = self.query();
            let matches = qc.matches(&query, root, s.as_bytes());

            let ranges = matches
                .flat_map(|query_match| query_match.captures)
                .map(|capture| capture.node.byte_range());

            ranges.collect()
        })
    }
}
