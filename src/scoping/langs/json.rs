use std::fmt::Debug;

use clap::ValueEnum;

use super::{Find, LanguageScoper, QuerySource, TSLanguage, TSQuery, TSQueryError};

/// A compiled query for the JSON language.
#[derive(Debug)]
pub struct CompiledQuery(super::CompiledQuery);

impl TryFrom<QuerySource> for CompiledQuery {
    type Error = TSQueryError;

    /// Create a new compiled query for the JSON language.
    ///
    /// # Errors
    ///
    /// See the concrete type of the [`TSQueryError`](tree_sitter::QueryError) variant for when this method errors.
    fn try_from(query: QuerySource) -> Result<Self, Self::Error> {
        let q = super::CompiledQuery::from_source(&tree_sitter_json::LANGUAGE.into(), &query)?;
        Ok(Self(q))
    }
}

impl From<PreparedQuery> for CompiledQuery {
    fn from(query: PreparedQuery) -> Self {
        Self(super::CompiledQuery::from_prepared_query(
            &tree_sitter_json::LANGUAGE.into(),
            query.as_string(),
        ))
    }
}

/// Prepared tree-sitter queries for JSON.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PreparedQuery {
    /// Comments (non-standard but widely supported)
    Comments,
    /// All JSON objects
    Objects,
    /// All JSON arrays
    Arrays,
    /// All string literals
    Strings,
    /// All number literals (both integers and floats)
    Numbers,
    /// Boolean literals (true and false)
    BooleanLiterals,
    /// Null values
    NullValues,
    /// Object key-value pairs
    ObjectKeyValuePairs,
    /// Array elements
    ArrayElements,
    /// String escape sequences
    StringEscapeSequences,
    /// All JSON values (catch-all for all value types)
    AllValues,
    /// Nested objects (objects within objects)
    NestedObjects,
    /// Nested arrays (arrays within arrays)
    NestedArrays,
    /// Mixed structures (heterogeneous arrays/objects)
    MixedStructures,
    /// Integer numbers specifically
    IntegerNumbers,
    /// Floating-point numbers specifically
    FloatNumbers,
    /// Double-quoted strings
    DoubleQuotedStrings,
    /// JSON top-level values
    TopLevelValues,
    /// Empty objects
    EmptyObjects,
}

impl PreparedQuery {
    /// Returns the tree-sitter query string for this prepared query.
    #[must_use]
    pub const fn as_string(self) -> &'static str {
        #[allow(clippy::match_same_arms)]
        match self {
            Self::Comments => "(comment) @comment",
Self::Objects => r"(object (pair key: (string) @property)) @object",
Self::Arrays => "(array (_value) @element) @array",
            Self::Strings => "(string) @string",
            Self::Numbers => "(number) @number",
            Self::BooleanLiterals => "(true) @boolean (false) @boolean",
            Self::NullValues => "(null) @null",
            Self::ObjectKeyValuePairs => "(object (pair) @pair) @object",
            Self::ArrayElements => "(array (_value) @value) @array",
            Self::StringEscapeSequences => "(escape_sequence) @escape",
            Self::AllValues => "(string) @value (number) @value (true) @value (false) @value (null) @value (object) @value (array) @value",
            Self::NestedObjects => "(object (pair value: (object) @nested)) @parent",
            Self::NestedArrays => "(array (array) @nested) @parent",
            Self::MixedStructures => r"(array (object) @object (array) @array (string) @string) @array",
Self::IntegerNumbers => "(number) @integer",
Self::FloatNumbers => "(number) @float",
Self::DoubleQuotedStrings => "(string) @string",
Self::TopLevelValues => "(document (number) @value (string) @value (true) @value (false) @value (null) @value (object) @value (array) @value) @document",
Self::EmptyObjects => "(object) @object",
        }
    }
}

impl Find for CompiledQuery {
    fn extensions(&self) -> &'static [&'static str] {
        &["json"]
    }
}

impl LanguageScoper for CompiledQuery {
    fn lang() -> TSLanguage
    where
        Self: Sized,
    {
        tree_sitter_json::LANGUAGE.into()
    }

    fn pos_query(&self) -> &TSQuery
    where
        Self: Sized,
    {
        &self.0.positive_query
    }

    fn neg_query(&self) -> Option<&TSQuery>
    where
        Self: Sized,
    {
        self.0.negative_query.as_ref()
    }
}