use std::fmt::Debug;

use clap::ValueEnum;

use super::{Find, LanguageScoper, RawQuery, TSLanguage, TSQuery, TSQueryError};

/// A compiled query for the TypeScript language.
#[derive(Debug)]
pub struct CompiledQuery(super::CompiledQuery);

impl TryFrom<RawQuery> for CompiledQuery {
    type Error = TSQueryError;

    /// Create a new compiled query for the TypeScript language.
    ///
    /// # Errors
    ///
    /// See the concrete type of the [`TSQueryError`](tree_sitter::QueryError)variant for when this method errors.
    fn try_from(query: RawQuery) -> Result<Self, Self::Error> {
        let q =
            super::CompiledQuery::new(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(), &query)?;
        Ok(Self(q))
    }
}

/// Prepared tree-sitter queries for TypeScript.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PreparedQuery {
    /// Comments.
    Comments,
    /// Strings (literal, template).
    Strings,
    /// Imports (module specifiers).
    Imports,
    /// Any `function` definitions.
    Function,
    /// `async function` definitions.
    AsyncFunction,
    /// Non-`async function` definitions.
    SyncFunction,
    /// Method definitions.
    Method,
    /// `constructor` method definitions.
    Constructor,
    /// `class` definitions.
    Class,
    /// `enum` definitions.
    Enum,
    /// `interface` definitions.
    Interface,
    /// `try`/`catch`/`finally` blocks.
    TryCatch,
    /// Variable declarations (`let`, `const`, `var`).
    VarDecl,
    /// `let` variable declarations.
    Let,
    /// `const` variable declarations.
    Const,
    /// `var` variable declarations.
    Var,
    /// Type (generic) parameters.
    TypeParams,
    /// Type alias declarations.
    TypeAlias,
    /// `namespace` blocks.
    Namespace,
    /// `export` blocks.
    Export,
}

impl From<PreparedQuery> for RawQuery {
    fn from(query: PreparedQuery) -> Self {
        Self(std::borrow::Cow::Borrowed(query.into()))
    }
}

impl From<PreparedQuery> for &'static str {
    fn from(value: PreparedQuery) -> Self {
        match value {
            PreparedQuery::Comments => "(comment) @comment",
            PreparedQuery::Imports => r"(import_statement source: (string (string_fragment) @sf))",
            PreparedQuery::Strings => "(string_fragment) @string",
            PreparedQuery::Function => "(function_declaration) @func",
            PreparedQuery::AsyncFunction => {
                r#"(
                    (function_declaration) @func (#match? @func "^async")
                )"#
            }
            PreparedQuery::SyncFunction => {
                r#"(
                    (function_declaration) @func (#not-match? @func "^async")
                )"#
            }
            PreparedQuery::Method => "(method_definition) @method",
            PreparedQuery::Constructor => {
                r#"(method_definition
                    name: (_) @name (#eq? @name "constructor")
                ) @constructor"#
            }
            PreparedQuery::Class => "(class_declaration) @class",
            PreparedQuery::Enum => "(enum_declaration) @enum",
            PreparedQuery::Interface => "(interface_declaration) @interface",
            PreparedQuery::TryCatch => "(try_statement) @try",
            PreparedQuery::VarDecl => "(variable_declarator) @var_decl",
            PreparedQuery::Let => {
                r#"(
                    (lexical_declaration) @let_decl (#match? @let_decl "^let ")
                )"#
            }
            PreparedQuery::Const => {
                r#"(
                    (lexical_declaration) @const_decl (#match? @const_decl "^const ")
                )"#
            }
            PreparedQuery::Var => {
                r#"(
                    (variable_declaration) @var_decl (#match? @var_decl "^var ")
                )"#
            }
            PreparedQuery::TypeParams => "(type_parameters) @type_parameters",
            PreparedQuery::TypeAlias => "(type_alias_declaration) @type_alias_declaration",
            PreparedQuery::Namespace => "(internal_module) @internal_module",
            PreparedQuery::Export => "(export_statement) @export",
        }
    }
}

impl LanguageScoper for CompiledQuery {
    fn lang() -> TSLanguage {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
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
        &["ts", "tsx"]
    }
}
