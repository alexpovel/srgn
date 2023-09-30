use super::{LanguageScopedViewBuildStep, TSLanguage, TSParser, TSQuery, TSQueryCursor};
use crate::scoping::{ScopedViewBuildStep, ScopedViewBuilder};
use clap::ValueEnum;
use std::fmt::Debug;
use strum::Display;
use tree_sitter::QueryError;

#[derive(Debug, Clone)]
pub struct Python {
    pub query: PythonQuery,
}

#[derive(Debug, Display, Clone)]
pub enum PythonQuery {
    Custom(CustomPythonQuery),
    Premade(PremadePythonQuery),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PremadePythonQuery {
    Comments,
    DocStrings,
}

impl From<&PremadePythonQuery> for TSQuery {
    fn from(value: &PremadePythonQuery) -> Self {
        TSQuery::new(
            Python::lang(),
            match value {
                PremadePythonQuery::Comments => "(comment) @comment",
                PremadePythonQuery::DocStrings => {
                    r#"
                    ((string) @docstring
                        (#match? @docstring "^\"\"\"")
                    )
                    "#
                }
            },
        )
        .expect("Premade queries to be valid")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomPythonQuery(String);

impl TryFrom<String> for CustomPythonQuery {
    type Error = QueryError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match TSQuery::new(tree_sitter_python::language(), &value) {
            Ok(_) => Ok(Self(value.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<&CustomPythonQuery> for TSQuery {
    fn from(value: &CustomPythonQuery) -> Self {
        TSQuery::new(tree_sitter_python::language(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl From<&PythonQuery> for TSQuery {
    fn from(value: &PythonQuery) -> Self {
        match value {
            PythonQuery::Custom(query) => query.into(),
            PythonQuery::Premade(query) => query.into(),
        }
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

            let mut qc = TSQueryCursor::new();
            let query: TSQuery = (&self.query).into();
            let matches = qc.matches(&query, root, s.as_bytes());

            let ranges = matches
                .flat_map(|query_match| query_match.captures)
                .map(|capture| capture.node.byte_range());

            ranges.collect()
        })
    }
}

impl LanguageScopedViewBuildStep for Python {
    fn lang() -> TSLanguage {
        tree_sitter_python::language()
    }

    fn parser() -> TSParser {
        let mut parser = TSParser::new();
        parser
            .set_language(Self::lang())
            .expect("Error loading Python grammar");

        parser
    }
}
