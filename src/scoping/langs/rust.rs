use std::fmt::Debug;

use clap::ValueEnum;
use const_format::formatcp;

use super::{
    Find, IGNORE, LanguageScoper, QuerySource, TSLanguage, TSQuery, TSQueryError, TreeSitterRegex,
};

/// A compiled query for the Rust language.
#[derive(Debug)]
pub struct CompiledQuery(super::CompiledQuery);

impl TryFrom<QuerySource> for CompiledQuery {
    type Error = TSQueryError;

    /// Create a new compiled query for the Rust language.
    ///
    /// # Errors
    ///
    /// See the concrete type of the [`TSQueryError`](tree_sitter::QueryError) variant for when this method errors.
    fn try_from(query: QuerySource) -> Result<Self, Self::Error> {
        let q = super::CompiledQuery::from_source(&tree_sitter_rust::LANGUAGE.into(), &query)?;
        Ok(Self(q))
    }
}

impl From<PreparedQuery> for CompiledQuery {
    fn from(query: PreparedQuery) -> Self {
        Self(super::CompiledQuery::from_prepared_query(
            &tree_sitter_rust::LANGUAGE.into(),
            &query.as_string(),
        ))
    }
}

/// Prepared tree-sitter queries for Rust.
#[derive(Debug, Clone, ValueEnum)]
pub enum PreparedQuery {
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
    /// `struct` definitions, where the struct name matches the provided pattern.
    #[value(skip)] // Non-unit enum variants need: https://github.com/clap-rs/clap/issues/2621
    StructNamed(TreeSitterRegex),
    /// `struct` definitions not marked `pub`.
    PrivStruct,
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
    /// `enum` definitions, where the enum name matches the provided pattern.
    #[value(skip)] // Non-unit enum variants need: https://github.com/clap-rs/clap/issues/2621
    EnumNamed(TreeSitterRegex),
    /// `enum` definitions not marked `pub`.
    PrivEnum,
    /// `enum` definitions marked `pub`.
    PubEnum,
    /// `enum` definitions marked `pub(crate)`.
    PubCrateEnum,
    /// `enum` definitions marked `pub(self)`.
    PubSelfEnum,
    /// `enum` definitions marked `pub(super)`.
    PubSuperEnum,
    /// Variant members of `enum` definitions
    EnumVariant,
    /// Function definitions.
    Fn,
    /// Function definitions, where the function name matches the provided pattern.
    #[value(skip)] // Non-unit enum variants need: https://github.com/clap-rs/clap/issues/2621
    FnNamed(TreeSitterRegex),
    /// Function definitions inside `impl` blocks (associated functions/methods).
    ImplFn,
    /// Function definitions not marked `pub`.
    PrivFn,
    /// Function definitions marked `pub`.
    PubFn,
    /// Function definitions marked `pub(crate)`.
    PubCrateFn,
    /// Function definitions marked `pub(self)`.
    PubSelfFn,
    /// Function definitions marked `pub(super)`.
    PubSuperFn,
    /// Function definitions marked `const`
    ConstFn,
    /// Function definitions marked `async`
    AsyncFn,
    /// Function definitions marked `unsafe`
    UnsafeFn,
    /// Function definitions marked `extern`
    ExternFn,
    /// Function definitions with attributes containing `test` (`#[test]`, `#[rstest]`,
    /// ...).
    TestFn,
    /// `trait` definitions.
    Trait,
    /// `trait` definitions, where the trait name matches the provided pattern.
    #[value(skip)] // Non-unit enum variants need: https://github.com/clap-rs/clap/issues/2621
    TraitNamed(TreeSitterRegex),
    /// `impl` blocks.
    Impl,
    /// `impl` blocks for types (`impl SomeType {}`).
    ImplType,
    /// `impl` blocks for traits on types (`impl SomeTrait for SomeType {}`).
    ImplTrait,
    /// `mod` blocks.
    Mod,
    /// `mod` blocks, where the module name matches the provided pattern.
    #[value(skip)] // Non-unit enum variants need: https://github.com/clap-rs/clap/issues/2621
    ModNamed(TreeSitterRegex),
    /// `mod tests` blocks.
    ModTests,
    /// Type definitions (`struct`, `enum`, `union`).
    TypeDef,
    /// Identifiers.
    Identifier,
    /// Identifiers for types.
    TypeIdentifier,
    /// Closure definitions.
    Closure,
    /// `unsafe` keyword usages (`unsafe fn`, `unsafe` blocks, `unsafe Trait`, `unsafe
    /// impl Trait`).
    Unsafe,
}

impl PreparedQuery {
    #[expect(clippy::too_many_lines)]
    fn as_string(&self) -> String {
        match self {
            Self::Comments => {
                r#"
                [
                    (line_comment)+ @line
                    (block_comment)
                    (#not-match? @line "^///")
                ]
                @comment
                "#
            }
            .into(),
            Self::DocComments => {
                r#"
                (
                    (line_comment)+ @line
                    (#match? @line "^//(/|!)")
                )
                "#
            }
            .into(),
            Self::Uses => {
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
            .into(),
            Self::Strings => "(string_content) @string".into(),
            Self::Attribute => "(attribute) @attribute".into(),
            Self::Struct => "(struct_item) @struct_item".into(),
            Self::StructNamed(pattern) => {
                format!(
                    r#"(
                        (struct_item
                            name: _ @name
                        )
                        (#match? @name "{pattern}")
                    ) @struct_item"#,
                )
            }
            Self::PrivStruct => {
                r"(struct_item
                    .
                    name: (type_identifier)
                ) @struct_item_without_visibility_modifier"
            }
            .into(),
            Self::PubStruct => {
                r#"(struct_item
                    (visibility_modifier) @vis
                    (#eq? @vis "pub")
                ) @struct_item"#
            }
            .into(),
            Self::PubCrateStruct => {
                r"(struct_item
                    (visibility_modifier (crate))
                ) @struct_item"
            }
            .into(),
            Self::PubSelfStruct => {
                r"(struct_item
                    (visibility_modifier (self))
                ) @struct_item"
            }
            .into(),
            Self::PubSuperStruct => {
                r"(struct_item
                    (visibility_modifier (super))
                ) @struct_item"
            }
            .into(),
            Self::Enum => "(enum_item) @enum_item".into(),
            Self::EnumNamed(pattern) => {
                format!(
                    r#"(
                        (enum_item
                            name: _ @name
                        )
                        (#match? @name "{pattern}")
                    ) @enum_item"#,
                )
            }
            Self::PrivEnum => {
                r"(enum_item
                    .
                    name: (type_identifier)
                ) @enum_item_without_visibility_modifier"
            }
            .into(),
            Self::PubEnum => {
                r#"(enum_item
                    (visibility_modifier) @vis
                    (#eq? @vis "pub")
                ) @enum_item"#
            }
            .into(),
            Self::PubCrateEnum => {
                r"(enum_item
                    (visibility_modifier (crate))
                ) @enum_item"
            }
            .into(),
            Self::PubSelfEnum => {
                r"(enum_item
                    (visibility_modifier (self))
                ) @enum_item"
            }
            .into(),
            Self::PubSuperEnum => {
                r"(enum_item
                    (visibility_modifier (super))
                ) @enum_item"
            }
            .into(),
            Self::EnumVariant => "(enum_variant) @enum_variant".into(),
            Self::Fn => "(function_item) @function_item".into(),
            Self::FnNamed(pattern) => {
                format!(
                    r#"(
                        (function_item
                            name: _ @name
                        )
                        (#match? @name "{pattern}")
                    ) @function_item"#,
                )
            }
            Self::ImplFn => {
                r"(impl_item
                    body: (_ (function_item) @function)
                )"
            }
            .into(),
            Self::PrivFn => {
                r"(function_item
                    .
                    name: (identifier)
                ) @function_item_without_visibility_modifier"
            }
            .into(),
            Self::PubFn => {
                r#"(function_item
                    (visibility_modifier) @vis
                    (#eq? @vis "pub")
                ) @function_item"#
            }
            .into(),
            Self::PubCrateFn => {
                r"(function_item
                    (visibility_modifier (crate))
                ) @function_item"
            }
            .into(),
            Self::PubSelfFn => {
                r"(function_item
                    (visibility_modifier (self))
                ) @function_item"
            }
            .into(),
            Self::PubSuperFn => {
                r"(function_item
                    (visibility_modifier (super))
                ) @function_item"
            }
            .into(),
            Self::ConstFn => {
                r#"(function_item
                    (function_modifiers) @funcmods
                    (#match? @funcmods "const")
                ) @function_item"#
            }
            .into(),
            Self::AsyncFn => {
                r#"(function_item
                    (function_modifiers) @funcmods
                    (#match? @funcmods "async")
                ) @function_item"#
            }
            .into(),
            Self::UnsafeFn => {
                r#"(function_item
                    (function_modifiers) @funcmods
                    (#match? @funcmods "unsafe")
                ) @function_item"#
            }
            .into(),
            Self::ExternFn => {
                r"(function_item
                    (function_modifiers (extern_modifier))
                ) @extern_function"
            }
            .into(),
            Self::TestFn => {
                // Any attribute which matches aka contains `test`, preceded or
                // followed by more attributes, eventually preceded by a function.
                // The anchors of `.` ensure nothing but the items we're after occur
                // in between.
                formatcp!(
                    "
                    (
                        (attribute_item)*
                        .
                        (attribute_item (attribute) @{0}.attr (#match? @{0}.attr \"test\"))
                        .
                        (attribute_item)*
                        .
                        (function_item) @func
                    )",
                    IGNORE
                )
            }
            .into(),
            Self::Trait => "(trait_item) @trait_item".into(),
            Self::TraitNamed(pattern) => {
                format!(
                    r#"(
                        (trait_item
                            name: _ @name
                        )
                        (#match? @name "{pattern}")
                    ) @trait_item"#,
                )
            }
            Self::Impl => "(impl_item) @impl_item".into(),
            Self::ImplType => {
                r"(impl_item
                    type: (_)
                    !trait
                ) @impl_item"
            }
            .into(),
            Self::ImplTrait => {
                r"(impl_item
                    trait: (_)
                    .
                    type: (_)
                ) @impl_item"
            }
            .into(),
            Self::Mod => "(mod_item) @mod_item".into(),
            Self::ModNamed(pattern) => {
                format!(
                    r#"(
                        (mod_item
                            name: _ @name
                        )
                        (#match? @name "{pattern}")
                    ) @mod_item"#,
                )
            }
            Self::ModTests => {
                r#"(mod_item
                    name: (identifier) @mod_name
                    (#eq? @mod_name "tests")
                ) @mod_tests
                "#
            }
            .into(),
            Self::TypeDef => {
                r"
                [
                    (struct_item)
                    (enum_item)
                    (union_item)
                ]
                @typedef
                "
            }
            .into(),
            Self::Identifier => "(identifier) @identifier".into(),
            Self::TypeIdentifier => "(type_identifier) @identifier".into(),
            Self::Closure => "(closure_expression) @closure".into(),
            Self::Unsafe => {
                r#"
                    [
                        (
                            (trait_item) @ti (#match? @ti "^unsafe")
                        )
                        (
                            (impl_item) @ii (#match? @ii "^unsafe")
                        )
                        (function_item
                            (function_modifiers) @funcmods
                            (#match? @funcmods "unsafe")
                        ) @function_item
                        (function_signature_item
                            (function_modifiers) @funcmods
                            (#match? @funcmods "unsafe")
                        ) @function_signature_item
                        (unsafe_block) @block
                    ] @unsafe
                "#
            }
            .into(),
        }
    }
}

impl LanguageScoper for CompiledQuery {
    fn lang() -> TSLanguage {
        tree_sitter_rust::LANGUAGE.into()
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
        &["rs"]
    }
}
