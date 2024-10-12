use std::fmt::Debug;

use clap::ValueEnum;
use const_format::formatcp;

use super::{Query, Kind, Language, LanguageScoper, TSLanguage, TSQuery};
use crate::find::Find;
use crate::scoping::langs::IGNORE;

/// A type used to make the generic `Language` struct specific to the C# language and
/// provides the appropriate `tree_sitter::Language` object.
#[derive(Clone, Copy, Debug)]
pub struct LangKind {}

impl Kind for LangKind {
    fn ts_lang() -> TSLanguage {
        tree_sitter_c_sharp::LANGUAGE.into()
    }
}

/// The C# language.
pub type CSharp = Language<LangKind>;

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

impl From<PreparedCSharpQuery> for Query<'static> {
    fn from(value: PreparedCSharpQuery) -> Self {
        let s = match value {
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
            PreparedCSharpQuery::Method => "(method_declaration) @method",
            PreparedCSharpQuery::VariableDeclaration => "(variable_declaration) @variable",
            PreparedCSharpQuery::Property => "(property_declaration) @property",
            PreparedCSharpQuery::Constructor => "(constructor_declaration) @constructor",
            PreparedCSharpQuery::Destructor => "(destructor_declaration) @destructor",
            PreparedCSharpQuery::Field => "(field_declaration) @field",
            PreparedCSharpQuery::Attribute => "(attribute) @attribute",
            PreparedCSharpQuery::Identifier => "(identifier) @identifier",
        };

        s.into()
    }
}

impl LanguageScoper for CSharp {
    fn lang() -> TSLanguage {
        tree_sitter_c_sharp::LANGUAGE.into()
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
