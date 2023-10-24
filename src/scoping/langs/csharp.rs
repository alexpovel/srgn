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
    /// Comments.
    ///
    /// Covers all comments, including XML doc comments and inline comments.
    Comments,
}

impl From<PremadeCSharpQuery> for TSQuery {
    fn from(value: PremadeCSharpQuery) -> Self {
        TSQuery::new(
            CSharp::lang(),
            match value {
                PremadeCSharpQuery::Comments => "(comment) @comment",
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
        self.scope_via_query(input)
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
