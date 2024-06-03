use super::{CodeQuery, Language, LanguageScoper, TSLanguage, TSQuery};
use crate::scoping::{ROScopes, Scoper};
use clap::ValueEnum;
use std::{fmt::Debug, str::FromStr};
use tree_sitter::QueryError;

/// The Rust language.
pub type Rust = Language<RustQuery>;
/// A query for Rust.
pub type RustQuery = CodeQuery<CustomRustQuery, PremadeRustQuery>;

/// Premade tree-sitter queries for Rust.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PremadeRustQuery {
    /// Comments (line and block styles; excluding doc comments; comment chars incl.).
    Comments,
    /// Doc comments (comment chars included).
    DocComments,
    /// Use statements (paths only; excl. `use`/`as`/`*`).
    Uses,
    /// Strings (regular, raw, byte; includes interpolation parts in format strings!).
    ///
    /// There is currently no support for an 'interpolation' type node in
    /// tree-sitter-rust (like there is in TypeScript and Python, for example).
    Strings,
}

impl From<PremadeRustQuery> for TSQuery {
    fn from(value: PremadeRustQuery) -> Self {
        TSQuery::new(
            Rust::lang(),
            match value {
                PremadeRustQuery::Comments => {
                    r#"
                    [
                        (line_comment)+ @line
                        (block_comment)
                        (#not-match? @line "^///")
                    ]
                    @comment
                    "#
                }
                PremadeRustQuery::DocComments => {
                    r#"
                    (
                        (line_comment)+ @line
                        (#match? @line "^///")
                    )
                    "#
                }
                PremadeRustQuery::Uses => {
                    r"
                        (scoped_identifier
                            path: [
                                (scoped_identifier)
                                (identifier)
                            ] @use)
                        (scoped_use_list
                            path: [
                                (scoped_identifier)
                                (identifier)
                            ] @use)
                        (use_wildcard (scoped_identifier) @use)
                    "
                }
                PremadeRustQuery::Strings => {
                    r"
                    [
                        (string_literal)
                        (raw_string_literal)
                    ]
                    @string
                    "
                }
            },
        )
        .expect("Premade queries to be valid")
    }
}

/// A custom tree-sitter query for Rust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomRustQuery(String);

impl FromStr for CustomRustQuery {
    type Err = QueryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TSQuery::new(Rust::lang(), s) {
            Ok(_) => Ok(Self(s.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<CustomRustQuery> for TSQuery {
    fn from(value: CustomRustQuery) -> Self {
        TSQuery::new(Rust::lang(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl Scoper for Rust {
    fn scope<'viewee>(&self, input: &'viewee str) -> ROScopes<'viewee> {
        ROScopes::from_raw_ranges(
            input,
            Self::scope_via_query(&mut self.query(), input).into(),
        )
    }
}

impl LanguageScoper for Rust {
    fn lang() -> TSLanguage {
        tree_sitter_rust::language()
    }

    fn query(&self) -> TSQuery {
        self.query.clone().into()
    }
}
