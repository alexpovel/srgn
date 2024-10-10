use std::fmt::Debug;
use std::str::FromStr;

use clap::ValueEnum;
use tree_sitter::QueryError;

use super::{CodeQuery, Language, LanguageScoper, TSLanguage, TSQuery};
use crate::find::Find;

/// The Typst language.
pub type Typst = Language<TypstQuery>;
/// A query for Typst.
pub type TypstQuery = CodeQuery<CustomTypstQuery, PreparedTypstQuery>;

/// Prepared tree-sitter queries for Typst.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PreparedTypstQuery {
    /// Comments (single- and multi-line).
    Comments,
    /// Strings (interpreted and raw; excluding struct tags).
    Strings,
    /// Text
    Text,
    /// Content
    Content,
}

impl From<PreparedTypstQuery> for TSQuery {
    fn from(value: PreparedTypstQuery) -> Self {
        Self::new(
            &Typst::lang(),
            match value {
                PreparedTypstQuery::Comments => "(comment) @comment",
                PreparedTypstQuery::Strings => "(string) @string",
                PreparedTypstQuery::Text => "(text) @text",
                PreparedTypstQuery::Content => "(content) @content",
            },
        )
        .expect("Prepared queries to be valid")
    }
}

/// A custom tree-sitter query for Typst.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomTypstQuery(String);

impl FromStr for CustomTypstQuery {
    type Err = QueryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TSQuery::new(&Typst::lang(), s) {
            Ok(_) => Ok(Self(s.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<CustomTypstQuery> for TSQuery {
    fn from(value: CustomTypstQuery) -> Self {
        Self::new(&Typst::lang(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl LanguageScoper for Typst {
    fn lang() -> TSLanguage {
        tree_sitter_typst::LANGUAGE.into()
    }

    fn pos_query(&self) -> &TSQuery {
        &self.positive_query
    }

    fn neg_query(&self) -> Option<&TSQuery> {
        self.negative_query.as_ref()
    }
}

impl Find for Typst {
    fn extensions(&self) -> &'static [&'static str] {
        &["typ"]
    }
}
