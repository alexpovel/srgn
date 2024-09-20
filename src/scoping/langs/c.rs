use std::fmt::Debug;
use std::str::FromStr;

use clap::ValueEnum;
use tree_sitter::QueryError;

use super::{CodeQuery, Language, LanguageScoper, TSLanguage, TSQuery};
use crate::find::Find;

/// The C language.
pub type C = Language<CQuery>;
/// A query for C.
pub type CQuery = CodeQuery<CustomCQuery, PreparedCQuery>;

/// Prepared tree-sitter queries for C.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PreparedCQuery {
    /// Comments (single- and multi-line).
    Comments,
    /// Strings.
    Strings,
    /// Includes.
    Includes,
    /// Type definitions.
    TypeDef,
    /// `enum` definitions.
    Enum,
    /// `struct` type definitions.
    Struct,
    /// Variable definitions.
    Variable,
    /// All functions usages (declarations and calls).
    Function,
    /// Function definitions.
    FunctionDef,
    /// Function declaration.
    FunctionDecl,
    /// `switch` blocks.
    Switch,
    /// `if` blocks.
    If,
    /// `for` blocks.
    For,
    /// `while` blocks.
    While,
    /// `do` blocks.
    Do,
    /// `union` blocks.
    Union,
    /// Identifier.
    Identifier,
    /// Declaration.
    Declaration,
    /// Call expression.
    CallExpression,
}

impl From<PreparedCQuery> for TSQuery {
    fn from(value: PreparedCQuery) -> Self {
        Self::new(
            &C::lang(),
            match value {
                PreparedCQuery::Comments => "(comment) @comment",
                PreparedCQuery::Strings => "[(string_literal) (system_lib_string)] @string",
                PreparedCQuery::Includes => "(preproc_include) @include",
                PreparedCQuery::TypeDef => "(type_definition) @typedef",
                PreparedCQuery::Enum => "(enum_specifier) @enum",
                PreparedCQuery::Struct => "(struct_specifier) @struct",
                PreparedCQuery::Variable => "(declaration) @var",
                PreparedCQuery::Function => {
                    "[(function_declarator (identifier)) (call_expression (identifier))] @function"
                }
                PreparedCQuery::FunctionDef => "(function_definition) @function_definition",
                PreparedCQuery::FunctionDecl => "(function_declarator) @function_decl",
                PreparedCQuery::Switch => "(switch_statement) @switch",
                PreparedCQuery::If => "(if_statement) @if",
                PreparedCQuery::For => "(for_statement) @for",
                PreparedCQuery::While => "(while_statement) @while",
                PreparedCQuery::Union => "(union_specifier) @union",
                PreparedCQuery::Do => "(do_statement) @do",
                PreparedCQuery::Identifier => "(identifier) @ident",
                PreparedCQuery::Declaration => "(declaration) @decl",
                PreparedCQuery::CallExpression => "(call_expression) @call",
            },
        )
        .expect("Prepared queries to be valid")
    }
}

/// A custom tree-sitter query for C.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomCQuery(String);

impl FromStr for CustomCQuery {
    type Err = QueryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TSQuery::new(&C::lang(), s) {
            Ok(_) => Ok(Self(s.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<CustomCQuery> for TSQuery {
    fn from(value: CustomCQuery) -> Self {
        Self::new(&C::lang(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl LanguageScoper for C {
    fn lang() -> TSLanguage {
        tree_sitter_c::LANGUAGE.into()
    }

    fn pos_query(&self) -> &TSQuery {
        &self.positive_query
    }

    fn neg_query(&self) -> Option<&TSQuery> {
        self.negative_query.as_ref()
    }
}

impl Find for C {
    fn extensions(&self) -> &'static [&'static str] {
        &["c", "h"]
    }
}
