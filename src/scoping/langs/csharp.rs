use super::{CodeQuery, Language, LanguageScoper, TSLanguage, TSQuery};
use crate::scoping::{ROScopes, Scoper};
use clap::ValueEnum;
use std::{fmt::Debug, str::FromStr};
use tree_sitter::QueryError;

/// The C# language.
pub type CSharp = Language<CSharpQuery>;
/// A query for C#.
pub type CSharpQuery = CodeQuery<CustomCSharpQuery, PremadeCSharpQuery>;

/// Premade tree-sitter queries for C#.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PremadeCSharpQuery {
    /// Comments (including XML, inline, doc comments).
    Comments,
    /// Strings (incl. verbatim, interpolated; incl. quotes, except for interpolated).
    ///
    /// Raw strings are [not yet
    /// supported](https://github.com/tree-sitter/tree-sitter-c-sharp/pull/240).
    Strings,
    /// `using` directives (including periods).
    Usings,
}

impl From<PremadeCSharpQuery> for TSQuery {
    fn from(value: PremadeCSharpQuery) -> Self {
        TSQuery::new(
            CSharp::lang(),
            match value {
                PremadeCSharpQuery::Comments => "(comment) @comment",
                PremadeCSharpQuery::Usings => {
                    r"(using_directive [(identifier) (qualified_name)] @import)"
                }
                PremadeCSharpQuery::Strings => {
                    r"
                    [
                        (interpolated_string_text)
                        (interpolated_verbatim_string_text)
                        (string_literal)
                        (verbatim_string_literal)
                    ]
                    @string
                    "
                }
            },
        )
        .expect("Premade queries to be valid")
    }
}

/// A custom tree-sitter query for C#.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomCSharpQuery(String);

impl FromStr for CustomCSharpQuery {
    type Err = QueryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TSQuery::new(CSharp::lang(), s) {
            Ok(_) => Ok(Self(s.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<CustomCSharpQuery> for TSQuery {
    fn from(value: CustomCSharpQuery) -> Self {
        TSQuery::new(CSharp::lang(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl Scoper for CSharp {
    fn scope<'viewee>(&self, input: &'viewee str) -> ROScopes<'viewee> {
        ROScopes::from_raw_ranges(
            input,
            Self::scope_via_query(&mut self.query(), input).into(),
        )
    }
}

impl LanguageScoper for CSharp {
    fn lang() -> TSLanguage {
        tree_sitter_c_sharp::language()
    }

    fn query(&self) -> TSQuery {
        self.query.clone().into()
    }
}
