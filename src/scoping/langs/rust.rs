use std::fmt::Debug;

use clap::ValueEnum;

use super::{Find, LanguageScoper, RawQuery, TSLanguage, TSQuery};

/// A compiled query for the Rust language.
#[derive(Debug)]
pub struct CompiledQuery(super::CompiledQuery);

impl CompiledQuery {
    /// Create a new compiled query for the Rust language.
    ///
    /// # Errors
    ///
    /// See the concrete type of the [`TSQueryError`](tree_sitter::QueryError) variant for when this method errors.
    pub fn new(query: &RawQuery<'_>) -> Result<Self, super::TSQueryError> {
        let q = super::CompiledQuery::new(&tree_sitter_rust::LANGUAGE.into(), query)?;
        Ok(Self(q))
    }
}

/// Prepared tree-sitter queries for Rust.
#[derive(Debug, Clone, Copy, ValueEnum)]
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
    /// `impl` blocks.
    Impl,
    /// `impl` blocks for types (`impl SomeType {}`).
    ImplType,
    /// `impl` blocks for traits on types (`impl SomeTrait for SomeType {}`).
    ImplTrait,
    /// `mod` blocks.
    Mod,
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

impl From<PreparedQuery> for RawQuery<'static> {
    fn from(query: PreparedQuery) -> Self {
        let s: &'static str = query.into();
        s.into()
    }
}

impl From<PreparedQuery> for &'static str {
    #[allow(clippy::too_many_lines)]
    fn from(query: PreparedQuery) -> Self {
        match query {
            PreparedQuery::Comments => include_str!("../../../data/queries/rust/comments.scm"),
            PreparedQuery::DocComments => {
                include_str!("../../../data/queries/rust/doc_comments.scm")
            }
            PreparedQuery::Uses => include_str!("../../../data/queries/rust/uses.scm"),
            PreparedQuery::Strings => include_str!("../../../data/queries/rust/strings.scm"),
            PreparedQuery::Attribute => include_str!("../../../data/queries/rust/attribute.scm"),
            PreparedQuery::Struct => include_str!("../../../data/queries/rust/struct.scm"),
            PreparedQuery::PrivStruct => include_str!("../../../data/queries/rust/priv_struct.scm"),
            PreparedQuery::PubStruct => include_str!("../../../data/queries/rust/pub_struct.scm"),
            PreparedQuery::PubCrateStruct => {
                include_str!("../../../data/queries/rust/pub_crate_struct.scm")
            }
            PreparedQuery::PubSelfStruct => {
                include_str!("../../../data/queries/rust/pub_self_struct.scm")
            }
            PreparedQuery::PubSuperStruct => {
                include_str!("../../../data/queries/rust/pub_super_struct.scm")
            }
            PreparedQuery::Enum => include_str!("../../../data/queries/rust/enum.scm"),
            PreparedQuery::PrivEnum => include_str!("../../../data/queries/rust/priv_enum.scm"),
            PreparedQuery::PubEnum => include_str!("../../../data/queries/rust/pub_enum.scm"),
            PreparedQuery::PubCrateEnum => {
                include_str!("../../../data/queries/rust/pub_crate_enum.scm")
            }
            PreparedQuery::PubSelfEnum => {
                include_str!("../../../data/queries/rust/pub_self_enum.scm")
            }
            PreparedQuery::PubSuperEnum => {
                include_str!("../../../data/queries/rust/pub_super_enum.scm")
            }
            PreparedQuery::EnumVariant => {
                include_str!("../../../data/queries/rust/enum_variant.scm")
            }
            PreparedQuery::Fn => include_str!("../../../data/queries/rust/fn.scm"),
            PreparedQuery::ImplFn => include_str!("../../../data/queries/rust/impl_fn.scm"),
            PreparedQuery::PrivFn => include_str!("../../../data/queries/rust/priv_fn.scm"),
            PreparedQuery::PubFn => include_str!("../../../data/queries/rust/pub_fn.scm"),
            PreparedQuery::PubCrateFn => {
                include_str!("../../../data/queries/rust/pub_crate_fn.scm")
            }
            PreparedQuery::PubSelfFn => include_str!("../../../data/queries/rust/pub_self_fn.scm"),
            PreparedQuery::PubSuperFn => {
                include_str!("../../../data/queries/rust/pub_super_fn.scm")
            }
            PreparedQuery::ConstFn => include_str!("../../../data/queries/rust/const_fn.scm"),
            PreparedQuery::AsyncFn => include_str!("../../../data/queries/rust/async_fn.scm"),
            PreparedQuery::UnsafeFn => include_str!("../../../data/queries/rust/unsafe_fn.scm"),
            PreparedQuery::ExternFn => include_str!("../../../data/queries/rust/extern_fn.scm"),
            PreparedQuery::TestFn => include_str!("../../../data/queries/rust/test_fn.scm"),
            PreparedQuery::Trait => include_str!("../../../data/queries/rust/trait.scm"),
            PreparedQuery::Impl => include_str!("../../../data/queries/rust/impl.scm"),
            PreparedQuery::ImplType => include_str!("../../../data/queries/rust/impl_type.scm"),
            PreparedQuery::ImplTrait => include_str!("../../../data/queries/rust/impl_trait.scm"),
            PreparedQuery::Mod => include_str!("../../../data/queries/rust/mod.scm"),
            PreparedQuery::ModTests => include_str!("../../../data/queries/rust/mod_tests.scm"),
            PreparedQuery::TypeDef => include_str!("../../../data/queries/rust/type_def.scm"),
            PreparedQuery::Identifier => include_str!("../../../data/queries/rust/identifier.scm"),
            PreparedQuery::TypeIdentifier => {
                include_str!("../../../data/queries/rust/type_identifier.scm")
            }
            PreparedQuery::Closure => include_str!("../../../data/queries/rust/closure.scm"),
            PreparedQuery::Unsafe => include_str!("../../../data/queries/rust/unsafe.scm"),
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
