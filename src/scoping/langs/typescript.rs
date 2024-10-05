use std::fmt::Debug;

use clap::ValueEnum;

use super::{CodeQuery, Find, Kind, Language, LanguageScoper, TSLanguage, TSQuery};

/// A type used to make the generic `Language` struct specific to the TypeScript language and
/// provides the appropriate `tree_sitter::Language` object.
#[derive(Clone, Copy, Debug)]
pub struct LangKind {}

impl Kind for LangKind {
    fn ts_lang() -> TSLanguage {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
    }
}

/// The TypeScript language.
pub type TypeScript = Language<LangKind>;

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

impl From<PreparedTypeScriptQuery> for CodeQuery<'static> {
    fn from(value: PreparedTypeScriptQuery) -> Self {
        let s = match value {
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
        };

        s.into()
    }
}

impl LanguageScoper for TypeScript {
    fn lang() -> TSLanguage {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
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
