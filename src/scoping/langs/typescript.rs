use super::{CodeQuery, Find, Language, LanguageScoper, TSLanguage, TSQuery};
use clap::ValueEnum;
use std::{fmt::Debug, str::FromStr};
use tree_sitter::QueryError;

/// The TypeScript language.
pub type TypeScript = Language<TypeScriptQuery>;
/// A query for TypeScript.
pub type TypeScriptQuery = CodeQuery<CustomTypeScriptQuery, PreparedTypeScriptQuery>;
/// Prepared tree-sitter queries for TypeScript.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PreparedTypeScriptQuery {
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
    /// `enum` definitions.
    Enum,
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
}

impl From<PreparedTypeScriptQuery> for TSQuery {
    fn from(value: PreparedTypeScriptQuery) -> Self {
        Self::new(
            &TypeScript::lang(),
            match value {
                PreparedTypeScriptQuery::Comments => "(comment) @comment",
                PreparedTypeScriptQuery::Imports => {
                    r"(import_statement source: (string (string_fragment) @sf))"
                }
                PreparedTypeScriptQuery::Strings => "(string_fragment) @string",
                PreparedTypeScriptQuery::Function => "(function_declaration) @func",
                PreparedTypeScriptQuery::AsyncFunction => {
                    r#"(
                        (function_declaration) @func (#match? @func "^async")
                    )"#
                }
                PreparedTypeScriptQuery::SyncFunction => {
                    r#"(
                        (function_declaration) @func (#not-match? @func "^async")
                    )"#
                }
                PreparedTypeScriptQuery::Enum => "(enum_declaration) @enum",
                PreparedTypeScriptQuery::TryCatch => "(try_statement) @try",
                PreparedTypeScriptQuery::VarDecl => "(variable_declarator) @var_decl",
                PreparedTypeScriptQuery::Let => {
                    r#"(
                        (lexical_declaration) @let_decl (#match? @let_decl "^let ")
                    )"#
                }
                PreparedTypeScriptQuery::Const => {
                    r#"(
                        (lexical_declaration) @const_decl (#match? @const_decl "^const ")
                    )"#
                }
                PreparedTypeScriptQuery::Var => {
                    r#"(
                        (variable_declaration) @var_decl (#match? @var_decl "^var ")
                    )"#
                }
            },
        )
        .expect("Prepared queries to be valid")
    }
}

/// A custom tree-sitter query for TypeScript.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomTypeScriptQuery(String);

impl FromStr for CustomTypeScriptQuery {
    type Err = QueryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TSQuery::new(&TypeScript::lang(), s) {
            Ok(_) => Ok(Self(s.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<CustomTypeScriptQuery> for TSQuery {
    fn from(value: CustomTypeScriptQuery) -> Self {
        Self::new(&TypeScript::lang(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl LanguageScoper for TypeScript {
    fn lang() -> TSLanguage {
        tree_sitter_typescript::language_typescript()
    }

    fn pos_query(&self) -> &TSQuery {
        &self.positive_query
    }

    fn neg_query(&self) -> Option<&TSQuery> {
        self.negative_query.as_ref()
    }
}

impl Find for TypeScript {
    fn extensions(&self) -> &'static [&'static str] {
        &["ts", "tsx"]
    }
}
