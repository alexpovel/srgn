use super::Scoper;
use crate::ranges::Ranges;
#[cfg(doc)]
use crate::scoping::scope::Scope::{In, Out};
use log::{debug, trace};
use std::str::FromStr;
pub use tree_sitter::{
    Language as TSLanguage, Parser as TSParser, Query as TSQuery, QueryCursor as TSQueryCursor,
};

/// C#.
pub mod csharp;
/// Go.
pub mod go;
/// Python.
pub mod python;
/// Rust.
pub mod rust;
/// TypeScript.
pub mod typescript;

/// Represents a (programming) language.
#[derive(Debug)]
pub struct Language<Q> {
    query: Q,
}

impl<Q> Language<Q> {
    /// Create a new language with the given associated query over it.
    pub fn new(query: Q) -> Self {
        Self { query }
    }
}

/// A query over a language, for scoping.
///
/// Parts hit by the query are [`In`] scope, parts not hit are [`Out`] of scope.
#[derive(Debug, Clone)]
pub enum CodeQuery<C, P>
where
    C: FromStr + Into<TSQuery>,
    P: Into<TSQuery>,
{
    /// A custom, user-defined query.
    Custom(C),
    /// A premade query.
    ///
    /// Availability depends on the language, respective languages features, and
    /// implementation in this crate.
    Premade(P),
}

impl<C, P> From<CodeQuery<C, P>> for TSQuery
where
    C: FromStr + Into<TSQuery>,
    P: Into<TSQuery>,
{
    fn from(value: CodeQuery<C, P>) -> Self {
        match value {
            CodeQuery::Custom(query) => query.into(),
            CodeQuery::Premade(query) => query.into(),
        }
    }
}

/// In a query, use this name to mark a capture to be ignored.
///
/// Useful for queries where tree-sitter doesn't natively support a fitting node type,
/// and a result is instead obtained by ignoring unwanted parts of bigger captures.
pub(super) const IGNORE: &str = "_SRGN_IGNORE";

/// A scoper for a language.
///
/// Functions much the same, but provides specific language-related functionality.
pub trait LanguageScoper: Scoper {
    /// The language's tree-sitter language.
    fn lang() -> TSLanguage;

    /// The language's tree-sitter query.
    fn query(&self) -> TSQuery;

    /// The language's tree-sitter parser.
    #[must_use]
    fn parser() -> TSParser {
        let mut parser = TSParser::new();
        parser
            .set_language(&Self::lang())
            .expect("Should be able to load language grammar and parser");

        parser
    }

    /// Scope the given input using the language's query.
    ///
    /// In principle, this is the same as [`Scoper::scope`].
    fn scope_via_query(query: &mut TSQuery, input: &str) -> Ranges<usize> {
        // tree-sitter is about incremental parsing, which we don't use here
        let old_tree = None;

        trace!("Parsing into AST: {:?}", input);

        let tree = Self::parser()
            .parse(input, old_tree)
            .expect("No language set in parser, or other unrecoverable error");

        let root = tree.root_node();
        debug!(
            "S expression of parsed source code is: {:?}",
            root.to_sexp()
        );

        let run = |query: &mut TSQuery| {
            trace!("Running query: {:?}", query);

            let mut qc = TSQueryCursor::new();
            let matches = qc.matches(query, root, input.as_bytes());

            let mut ranges: Ranges<usize> = matches
                .flat_map(|query_match| query_match.captures)
                .map(|capture| capture.node.byte_range())
                .collect();

            // ⚠️ tree-sitter queries with multiple captures will return them in some
            // mixed order (not ordered, and not merged), but we later rely on cleanly
            // ordered, non-overlapping ranges (a bit unfortunate we have to know about
            // that remote part over here).
            ranges.merge();
            trace!("Querying yielded ranges: {:?}", ranges);

            ranges
        };

        let ranges = run(query);

        let is_ignored = |name: &str| name.starts_with(IGNORE);
        let has_ignored_captures = query.capture_names().iter().any(|name| is_ignored(name));

        if has_ignored_captures {
            let ignored_ranges = {
                let acknowledged_captures = query
                    .capture_names()
                    .iter()
                    .filter(|name| !is_ignored(name))
                    .map(|s| String::from(*s))
                    .collect::<Vec<_>>();

                for name in acknowledged_captures {
                    trace!("Disabling capture for: {:?}", name);
                    query.disable_capture(&name);
                }

                debug!("Query has captures to ignore: running additional query");
                run(query)
            };

            let res = ranges - ignored_ranges;
            debug!("Ranges cleaned up after subtracting ignores: {:?}", res);

            res
        } else {
            ranges
        }
    }
}
