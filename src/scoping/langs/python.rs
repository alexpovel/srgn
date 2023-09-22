use std::fmt::{self, Debug, Formatter};

use super::{Language, Parser, Query, QueryCursor};
use crate::scoping::ScopedView;

pub trait Scoper {
    fn scope<'a>(&self, input: &'a str) -> ScopedView<'a>;
}

impl Debug for dyn Scoper {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scoper").finish()
    }
}

pub trait LanguageScoper: Scoper {
    fn lang() -> Language;
    fn parser() -> Parser;
}

#[derive(Debug)]
pub struct PythonScoper {
    query: Query,
}

impl PythonScoper {
    pub fn new(query: &str) -> Self {
        let query = Query::new(Self::lang(), query).expect("Invalid query.");

        Self { query }
    }
}

impl Scoper for PythonScoper {
    fn scope<'a>(&self, input: &'a str) -> ScopedView<'a> {
        // view.explode(|s| {
        // tree-sitter is about incremental parsing, which we don't use here
        let old_tree = None;

        let tree = Self::parser()
            .parse(input, old_tree)
            .expect("No language set in parser, or other unrecoverable error");
        let root = tree.root_node();

        let mut qc = QueryCursor::new();
        let matches = qc.matches(&self.query, root, input.as_bytes());

        let ranges = matches
            .flat_map(|query_match| query_match.captures)
            .map(|capture| capture.node.byte_range());

        ScopedView::from_raw(input, ranges)
    }
}

impl LanguageScoper for PythonScoper {
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
