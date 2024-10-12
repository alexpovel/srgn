use std::fmt::Debug;

use clap::ValueEnum;

use super::{LanguageScoper, RawQuery, TSLanguage, TSQuery};
use crate::find::Find;

/// A compiled query for the C language.
#[derive(Debug)]
pub struct CompiledQuery(super::CompiledQuery);

impl CompiledQuery {
    /// Create a new compiled query for the C language
    ///
    /// # Errors
    ///
    /// See the concrete type of the [`TSQueryError`] variant for when this method errors.
    pub fn new(query: &RawQuery<'_>) -> Result<Self, super::TSQueryError> {
        let q = super::CompiledQuery::new(&tree_sitter_c::LANGUAGE.into(), query)?;
        Ok(Self(q))
    }
}

/// Prepared tree-sitter queries for C.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PreparedQuery {
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

impl From<PreparedQuery> for RawQuery<'static> {
    fn from(value: PreparedQuery) -> Self {
        let s = match value {
            PreparedQuery::Comments => "(comment) @comment",
            PreparedQuery::Strings => "[(string_literal) (system_lib_string)] @string",
            PreparedQuery::Includes => "(preproc_include) @include",
            PreparedQuery::TypeDef => "(type_definition) @typedef",
            PreparedQuery::Enum => "(enum_specifier) @enum",
            PreparedQuery::Struct => "(struct_specifier) @struct",
            PreparedQuery::Variable => "(declaration) @var",
            PreparedQuery::Function => {
                "[(function_declarator (identifier)) (call_expression (identifier))] @function"
            }
            PreparedQuery::FunctionDef => "(function_definition) @function_definition",
            PreparedQuery::FunctionDecl => "(function_declarator) @function_decl",
            PreparedQuery::Switch => "(switch_statement) @switch",
            PreparedQuery::If => "(if_statement) @if",
            PreparedQuery::For => "(for_statement) @for",
            PreparedQuery::While => "(while_statement) @while",
            PreparedQuery::Union => "(union_specifier) @union",
            PreparedQuery::Do => "(do_statement) @do",
            PreparedQuery::Identifier => "(identifier) @ident",
            PreparedQuery::Declaration => "(declaration) @decl",
            PreparedQuery::CallExpression => "(call_expression) @call",
        };

        s.into()
    }
}

impl LanguageScoper for CompiledQuery {
    fn lang() -> TSLanguage {
        tree_sitter_c::LANGUAGE.into()
    }

    fn pos_query(&self) -> &TSQuery {
        &self.0.positive_query
    }

    fn neg_query(&self) -> Option<&TSQuery> {
        self.0.negative_query.as_ref()
    }
}

impl Find for CompiledQuery {
    fn extensions(&self) -> &'static [&'static str] {
        &["c", "h"]
    }
}
