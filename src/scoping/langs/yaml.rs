use std::fmt::Debug;

use clap::ValueEnum;

use super::{Find, LanguageScoper, QuerySource, TSLanguage, TSQuery, TSQueryError};

/// A compiled query for the YAML language.
#[derive(Debug)]
pub struct CompiledQuery(super::CompiledQuery);

/// Prepared queries for YAML files.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum PreparedQuery {
    /// String scalar nodes, including mapping keys and values.
    StringScalar,
    /// Integer scalar nodes.
    IntegerScalar,
    /// Float scalar nodes.
    FloatScalar,
    /// Boolean scalar nodes.
    BooleanScalar,
    /// Block sequence nodes.
    BlockSequence,
    /// Block mapping nodes.
    BlockMapping,
    /// Flow sequence nodes.
    FlowSequence,
    /// Flow mapping nodes.
    FlowMapping,
}

impl PreparedQuery {
    /// Returns the tree-sitter query string for this prepared query variant.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StringScalar => {
                r"(string_scalar) @string
(double_quote_scalar) @string
(single_quote_scalar) @string"
            }
            Self::IntegerScalar => r"(integer_scalar) @integer",
            Self::FloatScalar => r"(float_scalar) @float",
            Self::BooleanScalar => r"(boolean_scalar) @boolean",
            Self::BlockSequence => r"(block_sequence) @sequence",
            Self::BlockMapping => r"(block_mapping) @mapping",
            Self::FlowSequence => r"(flow_sequence) @sequence",
            Self::FlowMapping => r"(flow_mapping) @mapping",
        }
    }
}

impl From<PreparedQuery> for CompiledQuery {
    fn from(query: PreparedQuery) -> Self {
        Self(super::CompiledQuery::from_prepared_query(
            &tree_sitter_yaml::LANGUAGE.into(),
            query.as_str(),
        ))
    }
}

impl TryFrom<QuerySource> for CompiledQuery {
    type Error = TSQueryError;

    fn try_from(source: QuerySource) -> Result<Self, Self::Error> {
        let q = super::CompiledQuery::from_source(&tree_sitter_yaml::LANGUAGE.into(), &source)?;
        Ok(Self(q))
    }
}

impl LanguageScoper for CompiledQuery {
    fn lang() -> TSLanguage
    where
        Self: Sized,
    {
        tree_sitter_yaml::LANGUAGE.into()
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
        &["yaml", "yml"]
    }

    fn interpreters(&self) -> Option<&'static [&'static str]> {
        None
    }
}
