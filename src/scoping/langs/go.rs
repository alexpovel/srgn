use std::fmt::Debug;
use std::path::{Component, Path};

use clap::ValueEnum;
use const_format::formatcp;

use super::{CodeQuery, Kind, Language, LanguageScoper, TSLanguage, TSQuery};
use crate::find::Find;
use crate::scoping::langs::IGNORE;

/// A type used to make the generic `Language` struct specific to the Go language and
/// provides the appropriate `tree_sitter::Language` object.
#[derive(Clone, Copy, Debug)]
pub struct LangKind {}

impl Kind for LangKind {
    fn ts_lang() -> TSLanguage {
        tree_sitter_go::LANGUAGE.into()
    }
}

/// The Go language.
pub type Go = Language<LangKind>;

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
    /// Type alias assignments.
    TypeAlias,
    /// `struct` type definitions.
    Struct,
    /// `interface` type definitions.
    Interface,
    /// `const` specifications.
    Const,
    /// `var` specifications.
    Var,
    /// `func` definitions.
    Func,
    /// Method `func` definitions (`func (recv Recv) SomeFunc()`).
    Method,
    /// Free `func` definitions (`func SomeFunc()`).
    FreeFunc,
    /// `func init()` definitions.
    InitFunc,
    /// Type parameters (generics).
    TypeParams,
    /// `defer` blocks.
    Defer,
    /// `select` blocks.
    Select,
    /// `go` blocks.
    Go,
    /// `switch` blocks.
    Switch,
    /// Labeled statements.
    Labeled,
    /// `goto` statements.
    Goto,
    /// Struct tags.
    StructTags,
}

impl From<PreparedGoQuery> for CodeQuery<'static> {
    fn from(value: PreparedGoQuery) -> Self {
        let s = match value {
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
            PreparedGoQuery::Imports => r"(import_spec path: (interpreted_string_literal) @path)",
            PreparedGoQuery::TypeDef => r"(type_declaration) @type_decl",
            PreparedGoQuery::TypeAlias => r"(type_alias) @type_alias",
            PreparedGoQuery::Struct => {
                r"(type_declaration (type_spec type: (struct_type))) @struct"
            }
            PreparedGoQuery::Interface => {
                r"(type_declaration (type_spec type: (interface_type))) @interface"
            }
            PreparedGoQuery::Const => "(const_spec) @const",
            PreparedGoQuery::Var => "(var_spec) @var",
            PreparedGoQuery::Func => {
                r"
                [
                    (method_declaration)
                    (function_declaration)
                    (func_literal)
                ] @func"
            }
            PreparedGoQuery::Method => "(method_declaration) @method",
            PreparedGoQuery::FreeFunc => "(function_declaration) @free_func",
            PreparedGoQuery::InitFunc => {
                r#"(function_declaration
                    name: (identifier) @id (#eq? @id "init")
                ) @init_func"#
            }
            PreparedGoQuery::TypeParams => "(type_parameter_declaration) @type_params",
            PreparedGoQuery::Defer => "(defer_statement) @defer",
            PreparedGoQuery::Select => "(select_statement) @select",
            PreparedGoQuery::Go => "(go_statement) @go",
            PreparedGoQuery::Switch => "(expression_switch_statement) @switch",
            PreparedGoQuery::Labeled => "(labeled_statement) @labeled",
            PreparedGoQuery::Goto => "(goto_statement) @goto",
            PreparedGoQuery::StructTags => "(field_declaration tag: (raw_string_literal) @tag)",
        };

        s.into()
    }
}

impl LanguageScoper for Go {
    fn lang() -> TSLanguage {
        tree_sitter_go::LANGUAGE.into()
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

    fn is_path_invalid(&self, path: &Path) -> bool {
        for component in path.components() {
            if let Component::Normal(item) = component {
                // https://go.dev/ref/mod#vendoring
                if item.as_encoded_bytes() == b"vendor" {
                    return true;
                }
            }
        }

        false
    }
}
