use super::{CodeQuery, Language, LanguageScoper, TSLanguage, TSQuery};
use crate::{find::Find, scoping::langs::IGNORE};
use clap::ValueEnum;
use const_format::formatcp;
use std::{fmt::Debug, str::FromStr};
use tree_sitter::QueryError;

/// The Go language.
pub type Go = Language<GoQuery>;
/// A query for Go.
pub type GoQuery = CodeQuery<CustomGoQuery, PreparedGoQuery>;

/// Prepared tree-sitter queries for Go.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PreparedGoQuery {
    /// Comments (single- and multi-line).
    Comments,
    /// Strings (interpreted and raw; excluding struct tags).
    Strings,
    /// Imports.
    Imports,
    /// Type definitions.
    TypeDef,
    /// `struct` type definitions.
    Struct,
    /// `interface` type definitions.
    Interface,
    /// Struct tags.
    StructTags,
}

impl From<PreparedGoQuery> for TSQuery {
    fn from(value: PreparedGoQuery) -> Self {
        Self::new(
            &Go::lang(),
            match value {
                PreparedGoQuery::Comments => "(comment) @comment",
                PreparedGoQuery::Strings => {
                    formatcp!(
                        r"
                        [
                            (raw_string_literal)
                            (interpreted_string_literal)
                            (import_spec (interpreted_string_literal)) @{0}
                            (field_declaration tag: (raw_string_literal)) @{0}
                        ]
                        @string",
                        IGNORE
                    )
                }
                PreparedGoQuery::Imports => {
                    r"(import_spec path: (interpreted_string_literal) @path)"
                }
                PreparedGoQuery::TypeDef => r"(type_declaration) @type_decl",
                PreparedGoQuery::Struct => {
                    r"(type_declaration (type_spec type: (struct_type))) @struct"
                }
                PreparedGoQuery::Interface => {
                    r"(type_declaration (type_spec type: (interface_type))) @interface"
                }
                PreparedGoQuery::StructTags => "(field_declaration tag: (raw_string_literal) @tag)",
            },
        )
        .expect("Prepared queries to be valid")
    }
}

/// A custom tree-sitter query for Go.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomGoQuery(String);

impl FromStr for CustomGoQuery {
    type Err = QueryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TSQuery::new(&Go::lang(), s) {
            Ok(_) => Ok(Self(s.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<CustomGoQuery> for TSQuery {
    fn from(value: CustomGoQuery) -> Self {
        Self::new(&Go::lang(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl LanguageScoper for Go {
    fn lang() -> TSLanguage {
        tree_sitter_go::language()
    }

    fn pos_query(&self) -> &TSQuery {
        &self.positive_query
    }

    fn neg_query(&self) -> Option<&TSQuery> {
        self.negative_query.as_ref()
    }
}

impl Find for Go {
    fn extensions(&self) -> &'static [&'static str] {
        &["go"]
    }
}
