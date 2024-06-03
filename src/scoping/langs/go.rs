use super::{CodeQuery, Language, LanguageScoper, TSLanguage, TSQuery};
use crate::scoping::{langs::IGNORE, ROScopes, Scoper};
use clap::ValueEnum;
use const_format::concatcp;
use std::{fmt::Debug, str::FromStr};
use tree_sitter::QueryError;

/// The Go language.
pub type Go = Language<GoQuery>;
/// A query for Go.
pub type GoQuery = CodeQuery<CustomGoQuery, PremadeGoQuery>;

/// Premade tree-sitter queries for Go.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PremadeGoQuery {
    /// Comments (single- and multi-line).
    Comments,
    /// Strings (interpreted and raw; excluding struct tags).
    Strings,
    /// Imports.
    Imports,
    /// Struct tags.
    StructTags,
}

impl From<PremadeGoQuery> for TSQuery {
    fn from(value: PremadeGoQuery) -> Self {
        TSQuery::new(
            Go::lang(),
            match value {
                PremadeGoQuery::Comments => "(comment) @comment",
                PremadeGoQuery::Strings => {
                    concatcp!(
                        "
                    [
                        (raw_string_literal)
                        (interpreted_string_literal)
                        (import_spec (interpreted_string_literal) @",
                        IGNORE,
                        ")
                        (field_declaration tag: (raw_string_literal) @",
                        IGNORE,
                        ")
                    ]
                    @string"
                    )
                }
                PremadeGoQuery::Imports => {
                    r"(import_spec path: (interpreted_string_literal) @path)"
                }
                PremadeGoQuery::StructTags => "(field_declaration tag: (raw_string_literal) @tag)",
            },
        )
        .expect("Premade queries to be valid")
    }
}

/// A custom tree-sitter query for Go.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomGoQuery(String);

impl FromStr for CustomGoQuery {
    type Err = QueryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TSQuery::new(Go::lang(), s) {
            Ok(_) => Ok(Self(s.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<CustomGoQuery> for TSQuery {
    fn from(value: CustomGoQuery) -> Self {
        TSQuery::new(Go::lang(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl Scoper for Go {
    fn scope<'viewee>(&self, input: &'viewee str) -> ROScopes<'viewee> {
        ROScopes::from_raw_ranges(
            input,
            Self::scope_via_query(&mut self.query(), input).into(),
        )
    }
}

impl LanguageScoper for Go {
    fn lang() -> TSLanguage {
        tree_sitter_go::language()
    }

    fn query(&self) -> TSQuery {
        self.query.clone().into()
    }
}
