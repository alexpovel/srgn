use std::fmt::Debug;

use clap::ValueEnum;
use const_format::formatcp;

use super::{LanguageScoper, RawQuery, TSLanguage, TSQuery, TSQueryError};
use crate::find::Find;
use crate::scoping::langs::IGNORE;

/// A compiled query for the C# language.
#[derive(Debug)]
pub struct CompiledQuery(super::CompiledQuery);

impl TryFrom<RawQuery> for CompiledQuery {
    type Error = TSQueryError;

    /// Create a new compiled query for the C# language.
    ///
    /// # Errors
    ///
    /// See the concrete type of the [`TSQueryError`](tree_sitter::QueryError)variant for when this method errors.
    fn try_from(query: RawQuery) -> Result<Self, Self::Error> {
        let q = super::CompiledQuery::from_raw_query(&tree_sitter_c_sharp::LANGUAGE.into(), query)?;
        Ok(Self(q))
    }
}

impl Into<CompiledQuery> for PreparedQuery {
    fn into(self) -> CompiledQuery {
        let q =
            super::CompiledQuery::from_preparred_query(&tree_sitter_c_sharp::LANGUAGE.into(), self);
        CompiledQuery(q)
    }
}

/// Prepared tree-sitter queries for C#.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PreparedQuery {
    /// Comments (including XML, inline, doc comments).
    Comments,
    /// Strings (incl. verbatim, interpolated; incl. quotes, except for interpolated).
    ///
    /// Raw strings are [not yet
    /// supported](https://github.com/tree-sitter/tree-sitter-c-sharp/pull/240).
    Strings,
    /// `using` directives (including periods).
    Usings,
    /// `struct` definitions (in their entirety).
    Struct,
    /// `enum` definitions (in their entirety).
    Enum,
    /// `interface` definitions (in their entirety).
    Interface,
    /// `class` definitions (in their entirety).
    Class,
    /// Method definitions (in their entirety).
    Method,
    /// Variable declarations (in their entirety).
    VariableDeclaration,
    /// Property definitions (in their entirety).
    Property,
    /// Constructor definitions (in their entirety).
    Constructor,
    /// Destructor definitions (in their entirety).
    Destructor,
    /// Field definitions on types (in their entirety).
    Field,
    /// Attribute names.
    Attribute,
    /// Identifier names.
    Identifier,
}

impl super::PreparedQuery for PreparedQuery {
    type Query = CompiledQuery;

    fn as_str(self) -> &'static str {
        match self {
            PreparedQuery::Comments => "(comment) @comment",
            PreparedQuery::Usings => r"(using_directive [(identifier) (qualified_name)] @import)",
            PreparedQuery::Strings => {
                formatcp!(
                    r"
                    [
                        (interpolated_string_expression (interpolation) @{0})
                        (string_literal)
                        (raw_string_literal)
                        (verbatim_string_literal)
                    ]
                    @string
                    ",
                    IGNORE
                )
            }
            PreparedQuery::Struct => "(struct_declaration) @struct",
            PreparedQuery::Enum => "(enum_declaration) @enum",
            PreparedQuery::Interface => "(interface_declaration) @interface",
            PreparedQuery::Class => "(class_declaration) @class",
            PreparedQuery::Method => "(method_declaration) @method",
            PreparedQuery::VariableDeclaration => "(variable_declaration) @variable",
            PreparedQuery::Property => "(property_declaration) @property",
            PreparedQuery::Constructor => "(constructor_declaration) @constructor",
            PreparedQuery::Destructor => "(destructor_declaration) @destructor",
            PreparedQuery::Field => "(field_declaration) @field",
            PreparedQuery::Attribute => "(attribute) @attribute",
            PreparedQuery::Identifier => "(identifier) @identifier",
        }
    }

    fn into_compiled_query(self) -> Self::Query {
        let q =
            super::CompiledQuery::from_preparred_query(&tree_sitter_c_sharp::LANGUAGE.into(), self);
        CompiledQuery(q)
    }
}

impl LanguageScoper for CompiledQuery {
    fn lang() -> TSLanguage {
        tree_sitter_c_sharp::LANGUAGE.into()
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
        &["cs"]
    }
}
