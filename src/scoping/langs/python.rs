use std::fmt::Debug;

use super::{
    Language, LanguageScopedViewBuildStep, LanguageScoperError, Parser, Query, QueryCursor,
};
use crate::scoping::{ScopedViewBuildStep, ScopedViewBuilder};

#[derive(Debug)]
pub struct Python {
    query: Query,
}

impl TryFrom<&str> for Python {
    type Error = LanguageScoperError;

    fn try_from(query: &str) -> Result<Self, Self::Error> {
        let query = Query::new(Self::lang(), query)?;

        Ok(Self { query })
    }
}

impl ScopedViewBuildStep for Python {
    fn scope<'a>(&self, input: &'a str) -> ScopedViewBuilder<'a> {
        ScopedViewBuilder::new(input).explode_from_ranges(|s| {
            // tree-sitter is about incremental parsing, which we don't use here
            let old_tree = None;

            let tree = Self::parser()
                .parse(s, old_tree)
                .expect("No language set in parser, or other unrecoverable error");
            let root = tree.root_node();

            let mut qc = QueryCursor::new();
            let matches = qc.matches(&self.query, root, s.as_bytes());

            let ranges = matches
                .flat_map(|query_match| query_match.captures)
                .map(|capture| capture.node.byte_range());

            ranges.collect()
        })
    }
}

impl LanguageScopedViewBuildStep for Python {
    fn lang() -> Language {
        tree_sitter_python::language()
    }

    fn parser() -> Parser {
        let mut parser = Parser::new();
        parser
            .set_language(Self::lang())
            .expect("Error loading Python grammar");

        parser
    }
}
