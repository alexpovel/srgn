use super::{CodeQuery, Language, LanguageScoper, TSLanguage, TSQuery};
use crate::{find::Find, scoping::langs::IGNORE};
use clap::ValueEnum;
use const_format::formatcp;
use std::{fmt::Debug, str::FromStr};
use tree_sitter::QueryError;

/// The C# language.
pub type CSharp = Language<CSharpQuery>;
/// A query for C#.
pub type CSharpQuery = CodeQuery<CustomCSharpQuery, PreparedCSharpQuery>;

/// Prepared tree-sitter queries for C#.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PreparedCSharpQuery {
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
    /// Variable declarations (in their entirety).
    VariableDeclaration,
    /// Property definitions (in their entirety).
    Property,
    /// Field definitions on types (in their entirety).
    Field,
    /// Attribute names.
    Attribute,
}

impl From<PreparedCSharpQuery> for TSQuery {
    fn from(value: PreparedCSharpQuery) -> Self {
        Self::new(
            &CSharp::lang(),
            match value {
                PreparedCSharpQuery::Comments => "(comment) @comment",
                PreparedCSharpQuery::Usings => {
                    r"(using_directive [(identifier) (qualified_name)] @import)"
                }
                PreparedCSharpQuery::Strings => {
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
                PreparedCSharpQuery::Struct => "(struct_declaration) @struct",
                PreparedCSharpQuery::Enum => "(enum_declaration) @enum",
                PreparedCSharpQuery::Interface => "(interface_declaration) @interface",
                PreparedCSharpQuery::Class => "(class_declaration) @class",
                PreparedCSharpQuery::VariableDeclaration => "(variable_declaration) @variable",
                PreparedCSharpQuery::Property => "(property_declaration) @property",
                PreparedCSharpQuery::Field => "(field_declaration) @field",
                PreparedCSharpQuery::Attribute => "(attribute) @attribute",
            },
        )
        .expect("Prepared queries to be valid")
    }
}

/// A custom tree-sitter query for C#.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomCSharpQuery(String);

impl FromStr for CustomCSharpQuery {
    type Err = QueryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TSQuery::new(&CSharp::lang(), s) {
            Ok(_) => Ok(Self(s.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<CustomCSharpQuery> for TSQuery {
    fn from(value: CustomCSharpQuery) -> Self {
        Self::new(&CSharp::lang(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl LanguageScoper for CSharp {
    fn lang() -> TSLanguage {
        tree_sitter_c_sharp::language()
    }

    fn pos_query(&self) -> &TSQuery {
        &self.positive_query
    }

    fn neg_query(&self) -> Option<&TSQuery> {
        self.negative_query.as_ref()
    }
}

impl Find for CSharp {
    fn extensions(&self) -> &'static [&'static str] {
        &["cs"]
    }
}
