use std::fmt::Debug;
use std::path::{Component, Path};

use clap::ValueEnum;
use const_format::formatcp;

use super::{LanguageScoper, QuerySource, TSLanguage, TSQuery, TSQueryError, TreeSitterRegex};
use crate::find::Find;
use crate::scoping::langs::IGNORE;

/// A compiled query for the Go language.
#[derive(Debug)]
pub struct CompiledQuery(super::CompiledQuery);

impl TryFrom<QuerySource> for CompiledQuery {
    type Error = TSQueryError;

    /// Create a new compiled query for the Go language.
    ///
    /// # Errors
    ///
    /// See the concrete type of the [`TSQueryError`](tree_sitter::QueryError)variant for when this method errors.
    fn try_from(query: QuerySource) -> Result<Self, Self::Error> {
        let q = super::CompiledQuery::from_source(&tree_sitter_go::LANGUAGE.into(), &query)?;
        Ok(Self(q))
    }
}

impl From<PreparedQuery> for CompiledQuery {
    fn from(query: PreparedQuery) -> Self {
        Self(super::CompiledQuery::from_prepared_query(
            &tree_sitter_go::LANGUAGE.into(),
            &query.as_string(),
        ))
    }
}

/// Prepared tree-sitter queries for Go.
#[derive(Debug, Clone, ValueEnum)]
pub enum PreparedQuery {
    /// Comments (single- and multi-line).
    Comments,
    /// Strings (interpreted and raw; excluding struct tags).
    Strings,
    /// Imports.
    Imports,
    /// Expressions (all of them!).
    Expression,
    /// Type definitions.
    TypeDef,
    /// Type alias assignments.
    TypeAlias,
    /// `struct` type definitions.
    Struct,
    /// `struct` type definitions, where the struct name matches the provided pattern.
    #[value(skip)] // Non-unit enum variants need: https://github.com/clap-rs/clap/issues/2621
    StructNamed(TreeSitterRegex),
    /// `interface` type definitions.
    Interface,
    /// `interface` type definitions, where the interface name matches the provided pattern.
    #[value(skip)] // Non-unit enum variants need: https://github.com/clap-rs/clap/issues/2621
    InterfaceNamed(TreeSitterRegex),
    /// `const` specifications.
    Const,
    /// `var` specifications.
    Var,
    /// `func` definitions.
    Func,
    /// `func` definitions, where the function name matches the provided pattern.
    #[value(skip)] // Non-unit enum variants need: https://github.com/clap-rs/clap/issues/2621
    FuncNamed(TreeSitterRegex),
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

impl PreparedQuery {
    fn as_string(&self) -> String {
        match self {
            Self::Comments => "(comment) @comment".into(),
            Self::Strings => {
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
            .into(),
            Self::Imports => r"(import_spec path: (interpreted_string_literal) @path)".into(),
            Self::Expression => r"(_expression) @expr".into(),
            Self::TypeDef => r"(type_declaration) @type_decl".into(),
            Self::TypeAlias => r"(type_alias) @type_alias".into(),
            Self::Struct => r"(type_declaration (type_spec type: (struct_type))) @struct".into(),
            Self::StructNamed(pattern) => {
                format!(
                    r#"(
                        type_declaration(
                            type_spec
                                name: _ @name
                                type: (struct_type)
                        )
                        (#match? @name "{pattern}")
                    ) @struct"#
                )
            }
            Self::Interface => {
                r"(type_declaration (type_spec type: (interface_type))) @interface".into()
            }
            Self::InterfaceNamed(pattern) => {
                format!(
                    r#"(
                        type_declaration(
                            type_spec
                                name: _ @name
                                type: (interface_type)
                        )
                        (#match? @name "{pattern}")
                    ) @interface"#
                )
            }
            Self::Const => "(const_spec) @const".into(),
            Self::Var => "(var_spec) @var".into(),
            Self::Func => {
                r"
                [
                    (method_declaration)
                    (function_declaration)
                    (func_literal)
                ] @func"
            }
            .into(),
            Self::FuncNamed(pattern) => {
                format!(
                    r#"(
                        [
                            (method_declaration
                                name: _ @name
                            )
                            (function_declaration
                                name: _ @name
                            )
                        ]
                        (#match? @name "{pattern}")
                    ) @func"#
                )
            }
            Self::Method => "(method_declaration) @method".into(),
            Self::FreeFunc => "(function_declaration) @free_func".into(),
            Self::InitFunc => {
                r#"(function_declaration
                    name: (identifier) @id (#eq? @id "init")
                ) @init_func"#
            }
            .into(),
            Self::TypeParams => "(type_parameter_declaration) @type_params".into(),
            Self::Defer => "(defer_statement) @defer".into(),
            Self::Select => "(select_statement) @select".into(),
            Self::Go => "(go_statement) @go".into(),
            Self::Switch => "(expression_switch_statement) @switch".into(),
            Self::Labeled => "(labeled_statement) @labeled".into(),
            Self::Goto => "(goto_statement) @goto".into(),
            Self::StructTags => "(field_declaration tag: (raw_string_literal) @tag)".into(),
        }
    }
}

impl LanguageScoper for CompiledQuery {
    fn lang() -> TSLanguage {
        tree_sitter_go::LANGUAGE.into()
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
