use std::borrow::Cow;

use log::{debug, error, info, trace};
use tree_sitter::{
    Language as TSLanguage, Parser as TSParser, Query as TSQuery, QueryCursor as TSQueryCursor,
    QueryError as TSQueryError, StreamingIterator,
};

use super::Scoper;
use super::scope::RangesWithContext;
use crate::find::Find;
use crate::ranges::Ranges;
#[cfg(doc)]
use crate::scoping::{
    scope::Scope::{In, Out},
    view::ScopedViewBuilder,
};

/// C.
pub mod c;
/// C#.
pub mod csharp;
/// Go.
pub mod go;
/// Hashicorp Configuration Language
pub mod hcl;
/// Python.
pub mod python;
/// Rust.
pub mod rust;
/// TypeScript.
pub mod typescript;

/// A regular expression pattern as supported by tree-sitter.
///
/// While tree-sitter supports the `match` directives, concrete support [depends on
/// implementations](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/3-predicates-and-directives.html#admonition-info).
/// This type wraps over [what the tree-sitter Rust bindings
/// use](https://github.com/tree-sitter/tree-sitter/blob/e3db212b0b12a189010979b4c129a792009e8361/lib/binding_rust/lib.rs#L2675),
/// granting some improved type safety (things blow up before the regex is built by
/// tree-sitter, at which point we get less rich error feedback), for better validation
/// -- **at the risk of these types going out of sync with the upstream**.
#[derive(Debug, Clone)]
pub struct TreeSitterRegex(pub regex::bytes::Regex);

impl std::fmt::Display for TreeSitterRegex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Passing patterns into tree-sitter queries for `match?` requires escaping, cf.
        // https://tree-sitter.github.io/tree-sitter/using-parsers/queries/3-predicates-and-directives.html#the-match-predicate.
        let escaped = regex::escape(self.0.as_str());

        write!(f, "{escaped}")
    }
}

/// Represents query compiled for a (programming) language L.
#[derive(Debug)]
struct CompiledQuery {
    /// The *positive* query: it will be run against input and its results used for
    /// scoping.
    positive_query: TSQuery,
    /// The *negative* query: if present (if [`IGNORE`] is present) will be run and
    /// *subtracted* from the positive query.
    negative_query: Option<TSQuery>,
}

impl CompiledQuery {
    /// Create a new `CompiledQuery` from a `RawQuery`.
    ///
    /// # Errors
    ///
    /// See the concrete type of the [`TSQueryError`] variant for when this method errors.
    fn from_source(lang: &TSLanguage, query: &QuerySource) -> Result<Self, TSQueryError> {
        Self::from_str(lang, &query.0)
    }

    fn from_prepared_query(lang: &TSLanguage, query: &str) -> Self {
        Self::from_str(lang, query).unwrap_or_else(|qe| {
            error!("Failed to compile prepared query: {qe:?}");
            panic!("syntax of prepared queries should be validated by tests, and injection of regex be protected by {}", stringify!(TreeSitterRegex))
        })
    }

    fn from_str(lang: &TSLanguage, query: &str) -> Result<Self, TSQueryError> {
        let positive_query = TSQuery::new(lang, query)?;

        let is_ignored = |name: &str| name.starts_with(IGNORE);
        let has_ignored_captures = positive_query
            .capture_names()
            .iter()
            .any(|name| is_ignored(name));

        let negative_query = has_ignored_captures
            .then(|| {
                let mut query = TSQuery::new(lang, query)?;
                let acknowledged_captures = query
                    .capture_names()
                    .iter()
                    .filter(|name| !is_ignored(name))
                    .map(|s| String::from(*s))
                    .collect::<Vec<_>>();

                for name in acknowledged_captures {
                    trace!("Disabling capture for: {name:?}");
                    query.disable_capture(&name);
                }

                Ok(query)
            })
            .transpose()?;

        Ok(Self {
            positive_query,
            negative_query,
        })
    }
}

/// An uncompiled source for a query over a language, for scoping.
///
/// Parts hit by the query are [`In`] scope, parts not hit are [`Out`] of scope.
#[derive(Clone, Debug)]
pub struct QuerySource(Cow<'static, str>);

impl From<String> for QuerySource {
    fn from(s: String) -> Self {
        Self(s.into())
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
pub trait LanguageScoper: Scoper + Find + Send + Sync {
    /// The language's tree-sitter language.
    fn lang() -> TSLanguage
    where
        Self: Sized; // Exclude from trait object

    /// The language's *positive* tree-sitter query.
    ///
    /// Its results indicate items in scope.
    fn pos_query(&self) -> &TSQuery
    where
        Self: Sized; // Exclude from trait object

    /// The language's *negative* tree-sitter query.
    ///
    /// Its results will be *subtracted* from the positive ones.
    fn neg_query(&self) -> Option<&TSQuery>
    where
        Self: Sized; // Exclude from trait object

    /// The language's tree-sitter parser.
    #[must_use]
    fn parser() -> TSParser
    where
        Self: Sized, // Exclude from trait object
    {
        let mut parser = TSParser::new();
        parser
            .set_language(&Self::lang())
            .expect("Should be able to load language grammar and parser");

        parser
    }

    /// Scope the given input using the language's query.
    ///
    /// In principle, this is the same as [`Scoper::scope`].
    fn scope_via_query(&self, input: &str) -> Ranges<usize>
    where
        Self: Sized, // Exclude from trait object
    {
        // tree-sitter is about incremental parsing, which we don't use here
        let old_tree = None;

        trace!("Parsing into AST: {input:?}");

        let tree = Self::parser()
            .parse(input, old_tree)
            .expect("No language set in parser, or other unrecoverable error");

        let root = tree.root_node();
        debug!(
            "S expression of parsed source code is: {:?}",
            root.to_sexp()
        );

        let run = |query: &TSQuery| {
            trace!("Running query: {query:?}");

            let mut qc = TSQueryCursor::new();
            let mut matches = qc.matches(query, root, input.as_bytes());

            let mut ranges: Ranges<usize> = {
                // The size hint is hard-coded to 0 currently so there's no effect; use it
                // regardless as it might be useful in the future.
                let mut ranges = Vec::with_capacity(matches.size_hint().1.unwrap_or_default());

                while let Some(m) = matches.next() {
                    for capture in m.captures {
                        ranges.push(capture.node.byte_range());
                    }
                }

                ranges.into_iter().collect()
            };

            // ⚠️ tree-sitter queries with multiple captures will return them in some
            // mixed order (not ordered, and not merged), but we later rely on cleanly
            // ordered, non-overlapping ranges (a bit unfortunate we have to know about
            // that remote part over here).
            ranges.merge();
            trace!("Querying yielded ranges: {ranges:?}");

            ranges
        };

        let ranges = run(self.pos_query());
        match &self.neg_query() {
            Some(nq) => ranges - run(nq),
            None => ranges,
        }
    }
}

impl<T> Scoper for T
where
    T: LanguageScoper,
{
    fn scope_raw<'viewee>(&self, input: &'viewee str) -> RangesWithContext<'viewee> {
        self.scope_via_query(input).into()
    }
}

impl Scoper for Box<dyn LanguageScoper> {
    fn scope_raw<'viewee>(&self, input: &'viewee str) -> RangesWithContext<'viewee> {
        self.as_ref().scope_raw(input)
    }
}

impl Scoper for &[Box<dyn LanguageScoper>] {
    /// Allows *multiple* scopers to be applied all at once.
    ///
    /// They are OR'd together in the sense that if *any* of the scopers hit, a
    /// position/range is considered in scope. In some sense, this is the opposite of
    /// [`ScopedViewBuilder::explode`], which is subtractive.
    fn scope_raw<'viewee>(&self, input: &'viewee str) -> RangesWithContext<'viewee> {
        trace!("Scoping many scopes: {input:?}");

        if self.is_empty() {
            trace!("Short-circuiting: self is empty, nothing to scope.");
            return vec![(0..input.len(), None)].into_iter().collect();
        }

        // This is slightly leaky in that it drops down to a more 'primitive' layer and
        // uses `Ranges`.
        let mut ranges: Ranges<usize> = self
            .iter()
            .flat_map(|s| s.scope_raw(input))
            .map(|(range, ctx)| {
                assert!(
                    ctx.is_none(),
                    "When language scoping runs, no contexts exist yet."
                );
                range
            })
            .collect();
        ranges.merge();
        info!("New ranges after scoping many: {ranges:?}");

        let ranges: RangesWithContext<'_> = ranges.into_iter().map(|r| (r, None)).collect();

        ranges
    }
}
