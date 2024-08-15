use super::{CodeQuery, Find, Language, LanguageScoper, TSLanguage, TSQuery};
use clap::ValueEnum;
use std::{fmt::Debug, str::FromStr};
use tree_sitter::QueryError;

/// The Rust language.
pub type Rust = Language<RustQuery>;
/// A query for Rust.
pub type RustQuery = CodeQuery<CustomRustQuery, PreparedRustQuery>;

/// Prepared tree-sitter queries for Rust.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PreparedRustQuery {
    /// Comments (line and block styles; excluding doc comments; comment chars incl.).
    Comments,
    /// Doc comments (comment chars included).
    DocComments,
    /// Use statements (paths only; excl. `use`/`as`/`*`).
    Uses,
    /// Strings (regular, raw, byte; includes interpolation parts in format strings!).
    ///
    /// There is currently no support for an 'interpolation' type node in
    /// tree-sitter-rust (like there is in TypeScript and Python, for example).
    Strings,
    /// Attributes like `#[attr]`.
    Attribute,
    /// `struct` definitions.
    Struct,
    /// `struct` definitions marked `pub`.
    PubStruct,
    /// `struct` definitions marked `pub(crate)`.
    PubCrateStruct,
    /// `struct` definitions marked `pub(self)`.
    PubSelfStruct,
    /// `struct` definitions marked `pub(super)`.
    PubSuperStruct,
    /// `enum` definitions.
    Enum,
    /// `enum` definitions marked `pub`.
    PubEnum,
    /// `enum` definitions marked `pub(crate)`.
    PubCrateEnum,
    /// `enum` definitions marked `pub(self)`.
    PubSelfEnum,
    /// `enum` definitions marked `pub(super)`.
    PubSuperEnum,
    /// Function definitions.
    Fn,
    /// Function definitions marked `pub`.
    PubFn,
    /// Function definitions marked `pub(crate)`.
    PubCrateFn,
    /// Function definitions marked `pub(self)`.
    PubSelfFn,
    /// Function definitions marked `pub(super)`.
    PubSuperFn,
    /// `mod` blocks.
    Mod,
    /// `mod tests` blocks.
    ModTests,
}

impl From<PreparedRustQuery> for TSQuery {
    #[allow(clippy::too_many_lines)]
    fn from(value: PreparedRustQuery) -> Self {
        Self::new(
            &Rust::lang(),
            match value {
                PreparedRustQuery::Comments => {
                    r#"
                    [
                        (line_comment)+ @line
                        (block_comment)
                        (#not-match? @line "^///")
                    ]
                    @comment
                    "#
                }
                PreparedRustQuery::DocComments => {
                    r#"
                    (
                        (line_comment)+ @line
                        (#match? @line "^//(/|!)")
                    )
                    "#
                }
                PreparedRustQuery::Uses => {
                    // Match any (wildcard `_`) `argument`, which includes:
                    //
                    // - `scoped_identifier`
                    // - `scoped_use_list`
                    // - `use_wildcard`
                    // - `use_as_clause`
                    //
                    // all at once.
                    r"
                    [
                        (use_declaration
                            argument: (_) @use
                        )
                    ]
                    "
                }
                PreparedRustQuery::Strings => "(string_content) @string",
                PreparedRustQuery::Attribute => "(attribute) @attribute",
                PreparedRustQuery::Struct => "(struct_item) @struct_item",
                PreparedRustQuery::PubStruct => {
                    r#"(struct_item
                        (visibility_modifier) @vis
                        (#eq? @vis "pub")
                    ) @struct_item"#
                }
                PreparedRustQuery::PubCrateStruct => {
                    r"(struct_item
                        (visibility_modifier (crate))
                    ) @struct_item"
                }
                PreparedRustQuery::PubSelfStruct => {
                    r"(struct_item
                        (visibility_modifier (self))
                    ) @struct_item"
                }
                PreparedRustQuery::PubSuperStruct => {
                    r"(struct_item
                        (visibility_modifier (super))
                    ) @struct_item"
                }
                PreparedRustQuery::Enum => "(enum_item) @enum_item",
                PreparedRustQuery::PubEnum => {
                    r#"(enum_item
                        (visibility_modifier) @vis
                        (#eq? @vis "pub")
                    ) @enum_item"#
                }
                PreparedRustQuery::PubCrateEnum => {
                    r"(enum_item
                        (visibility_modifier (crate))
                    ) @enum_item"
                }
                PreparedRustQuery::PubSelfEnum => {
                    r"(enum_item
                        (visibility_modifier (self))
                    ) @enum_item"
                }
                PreparedRustQuery::PubSuperEnum => {
                    r"(enum_item
                        (visibility_modifier (super))
                    ) @enum_item"
                }
                PreparedRustQuery::Fn => "(function_item) @function_item",
                PreparedRustQuery::PubFn => {
                    r#"(function_item
                        (visibility_modifier) @vis
                        (#eq? @vis "pub")
                    ) @function_item"#
                }
                PreparedRustQuery::PubCrateFn => {
                    r"(function_item
                        (visibility_modifier (crate))
                    ) @function_item"
                }
                PreparedRustQuery::PubSelfFn => {
                    r"(function_item
                        (visibility_modifier (self))
                    ) @function_item"
                }
                PreparedRustQuery::PubSuperFn => {
                    r"(function_item
                        (visibility_modifier (super))
                    ) @function_item"
                }
                PreparedRustQuery::Mod => "(mod_item) @mod_item",
                PreparedRustQuery::ModTests => {
                    r#"(mod_item
                        name: (identifier) @mod_name
                        (#eq? @mod_name "tests")
                    ) @mod_tests
                    "#
                }
            },
        )
        .expect("Prepared queries to be valid")
    }
}

/// A custom tree-sitter query for Rust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomRustQuery(String);

impl FromStr for CustomRustQuery {
    type Err = QueryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TSQuery::new(&Rust::lang(), s) {
            Ok(_) => Ok(Self(s.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<CustomRustQuery> for TSQuery {
    fn from(value: CustomRustQuery) -> Self {
        Self::new(&Rust::lang(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl LanguageScoper for Rust {
    fn lang() -> TSLanguage {
        tree_sitter_rust::language()
    }

    fn pos_query(&self) -> &TSQuery {
        &self.positive_query
    }

    fn neg_query(&self) -> Option<&TSQuery> {
        self.negative_query.as_ref()
    }
}

impl Find for Rust {
    fn extensions(&self) -> &'static [&'static str] {
        &["rs"]
    }
}
