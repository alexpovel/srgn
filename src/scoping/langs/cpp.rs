use std::fmt::Debug;
use std::str::FromStr;

use clap::ValueEnum;
use tree_sitter::QueryError;

use super::{CodeQuery, Language, LanguageScoper, TSLanguage, TSQuery};
use crate::find::Find;

/// The C++ language.
pub type Cpp = Language<CppQuery>;
/// A query for C++.
pub type CppQuery = CodeQuery<CustomCppQuery, PreparedCppQuery>;

/// Prepared tree-sitter queries for C++.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PreparedCppQuery {
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
    /// `class` definitions.
    Class,
    /// `namespace` definitions.
    Namespace,
    /// `using` namespace declarations.
    UsingNamespace,
    /// `template` declarations.
    Template,
    /// Field declarations.
    FieldDecl,
    /// Variable definitions.
    Variable,
    /// All functions usages (declarations and calls).
    Function,
    /// Function definitions.
    FunctionDef,
    /// Function declaration.
    FunctionDecl,
    /// Lambda
    Lambda,
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
    /// `try` blocks.
    Try,
    /// Identifier.
    Identifier,
    /// Declaration.
    Declaration,
    /// Call expression.
    CallExpression,
}

impl From<PreparedCppQuery> for TSQuery {
    fn from(value: PreparedCppQuery) -> Self {
        Self::new(
            &Cpp::lang(),
            match value {
                PreparedCppQuery::Comments => "(comment) @comment",
                PreparedCppQuery::Strings => "[(string_literal) (system_lib_string)] @string",
                PreparedCppQuery::Includes => "(preproc_include) @include",
                PreparedCppQuery::TypeDef => "(type_definition) @typedef",
                PreparedCppQuery::Enum => "(enum_specifier) @enum",
                PreparedCppQuery::Struct => "(struct_specifier) @struct",
                PreparedCppQuery::Class => "(class_specifier) @class",
                PreparedCppQuery::Namespace => "(namespace_definition) @namespace",
                PreparedCppQuery::UsingNamespace => "(using_declaration) @using",
                PreparedCppQuery::Template => "(template_declaration) @template",
                PreparedCppQuery::FieldDecl => "(field_declaration) @field_decl",
                PreparedCppQuery::Variable => "(declaration) @var",
                PreparedCppQuery::Function => {
                    "[(function_declarator (identifier)) (call_expression (identifier))] @function"
                }
                PreparedCppQuery::FunctionDef => "(function_definition) @function_definition",
                PreparedCppQuery::FunctionDecl => "(function_declarator) @function_decl",
                PreparedCppQuery::Lambda => "(lambda_expression) @lambda",
                PreparedCppQuery::Switch => "(switch_statement) @switch",
                PreparedCppQuery::If => "(if_statement) @if",
                PreparedCppQuery::For => "[(for_statement) (for_range_loop)] @for",
                PreparedCppQuery::While => "(while_statement) @while",
                PreparedCppQuery::Union => "(union_specifier) @union",
                PreparedCppQuery::Try => "(try_statement) @try",
                PreparedCppQuery::Do => "(do_statement) @do",
                PreparedCppQuery::Identifier => "(identifier) @ident",
                PreparedCppQuery::Declaration => "(declaration) @decl",
                PreparedCppQuery::CallExpression => "(call_expression) @call",
            },
        )
        .expect("Prepared queries to be valid")
    }
}

/// A custom tree-sitter query for C++.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomCppQuery(String);

impl FromStr for CustomCppQuery {
    type Err = QueryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TSQuery::new(&Cpp::lang(), s) {
            Ok(_) => Ok(Self(s.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<CustomCppQuery> for TSQuery {
    fn from(value: CustomCppQuery) -> Self {
        Self::new(&Cpp::lang(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl LanguageScoper for Cpp {
    fn lang() -> TSLanguage {
        tree_sitter_cpp::LANGUAGE.into()
    }

    fn pos_query(&self) -> &TSQuery {
        &self.positive_query
    }

    fn neg_query(&self) -> Option<&TSQuery> {
        self.negative_query.as_ref()
    }
}

impl Find for Cpp {
    fn extensions(&self) -> &'static [&'static str] {
        &["cpp", "hpp"]
    }
}
