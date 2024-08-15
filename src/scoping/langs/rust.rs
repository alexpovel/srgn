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
    /// Function definitions.
    Fn,
    /// Function definitions marked `pub`.
    PubFn,
    /// `mod` blocks.
    Mod,
    /// `mod tests` blocks.
    ModTests,
}

impl From<PreparedRustQuery> for TSQuery {
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
                PreparedRustQuery::Fn => "(function_item) @function_item",
                PreparedRustQuery::PubFn => {
                    r#"(function_item
                        (visibility_modifier) @vis
                        (#eq? @vis "pub")
                    ) @function_item"#
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
