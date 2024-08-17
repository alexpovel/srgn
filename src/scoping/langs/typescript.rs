use std::fmt::Debug;
use std::str::FromStr;

use clap::ValueEnum;
use tree_sitter::QueryError;

use super::{CodeQuery, Find, Language, LanguageScoper, TSLanguage, TSQuery};

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
                PreparedTypeScriptQuery::Method => "(method_definition) @method",
                PreparedTypeScriptQuery::Constructor => {
                    r#"(method_definition
                        name: (_) @name (#eq? @name "constructor")
                    ) @constructor"#
                }
                PreparedTypeScriptQuery::Class => "(class_declaration) @class",
                PreparedTypeScriptQuery::Enum => "(enum_declaration) @enum",
                PreparedTypeScriptQuery::Interface => "(interface_declaration) @interface",
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
                PreparedTypeScriptQuery::TypeParams => "(type_parameters) @type_parameters",
                PreparedTypeScriptQuery::TypeAlias => {
                    "(type_alias_declaration) @type_alias_declaration"
                }
                PreparedTypeScriptQuery::Namespace => "(internal_module) @internal_module",
                PreparedTypeScriptQuery::Export => "(export_statement) @export",
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
