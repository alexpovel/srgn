use super::{
    LanguageScopedViewBuildStep, LanguageScoperError, TSLanguage, TSParser, TSQuery, TSQueryCursor,
};
use crate::scoping::{ScopedViewBuildStep, ScopedViewBuilder};
use clap::{Parser, ValueEnum};
use enum_iterator::Sequence;
use std::fmt::Debug;
use strum::{Display, EnumString};
use tree_sitter::QueryError;

#[derive(Debug, Clone)]
pub struct Python {
    pub query: PythonQuery,
}

// pub trait FromRawQuery {
//     fn try_from_raw_query(query: &str) -> Result<Self, LanguageScoperError>
//     where
//         Self: Sized;

//     fn try_from_raw_premade_query(query: &str) -> Result<Self, LanguageScoperError>
//     where
//         Self: Sized;
// }

// impl FromRawQuery for Python {
//     fn try_from_raw_query(
//         query: &str,
//         // query: impl TryInto<MyBullshitQuery, Error = LanguageScoperError>,
//     ) -> Result<Self, LanguageScoperError> {
//         Ok(Self {
//             query: PythonQuery::Custom(query.try_into()?),
//         })
//     }

//     fn try_from_raw_premade_query(query: &str) -> Result<Self, LanguageScoperError> {
//         let x = query
//             .try_into()
//             .map_err(|_| LanguageScoperError::NoSuchPremadeQuery(query.to_string()))?;

//         Ok(Self {
//             query: PythonQuery::Premade(x),
//         })
//     }
// }

#[derive(Debug, Display, Clone)]
pub enum PythonQuery {
    Custom(CustomPythonQuery),
    Premade(PremadePythonQuery),
}

#[derive(Debug, Clone, Copy, EnumString, Display, Sequence, ValueEnum)]
#[strum(serialize_all = "kebab-case")]
pub enum PremadePythonQuery {
    Comments,
    DocStrings,
}

impl From<&PremadePythonQuery> for TSQuery {
    fn from(value: &PremadePythonQuery) -> Self {
        TSQuery::new(
            Python::lang(),
            match value {
                PremadePythonQuery::Comments => "(comment) @comment",
                PremadePythonQuery::DocStrings => {
                    r#"
                    ((string) @docstring
                        (#match? @docstring "^\"\"\"")
                    )
                    "#
                }
            },
        )
        .expect("Premade queries to be valid")
    }
}

#[derive(Debug, Clone)]
pub struct CustomPythonQuery(String);

// impl TryFrom<&str> for CustomPythonQuery {
//     type Error = LanguageScoperError;

//     fn try_from(value: &str) -> Result<Self, Self::Error> {
//         match TSQuery::new(tree_sitter_python::language(), value) {
//             Ok(_) => Ok(Self(value.to_string())),
//             Err(e) => Err(e.into()),
//         }
//     }
// }

impl TryFrom<String> for CustomPythonQuery {
    type Error = QueryError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match TSQuery::new(tree_sitter_python::language(), &value) {
            Ok(_) => Ok(Self(value.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<&CustomPythonQuery> for TSQuery {
    fn from(value: &CustomPythonQuery) -> Self {
        TSQuery::new(tree_sitter_python::language(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl From<&PythonQuery> for TSQuery {
    fn from(value: &PythonQuery) -> Self {
        match value {
            PythonQuery::Custom(query) => query.into(),
            PythonQuery::Premade(query) => query.into(),
        }
    }
}

impl ScopedViewBuildStep for Python {
    fn scope<'a>(&self, input: &'a str) -> ScopedViewBuilder<'a> {
        ScopedViewBuilder::new(input).explode_from_ranges(|s| {
            // tree-sitter is about incremental parsing, which we don't use here
            let old_tree = None;

            let tree = Self::parser()
                .parse(s, old_tree)
                .expect("No language set in parser, or other unrecoverable error");
            let root = tree.root_node();

            let mut qc = TSQueryCursor::new();
            let query: TSQuery = (&self.query).into();
            let matches = qc.matches(&query, root, s.as_bytes());

            let ranges = matches
                .flat_map(|query_match| query_match.captures)
                .map(|capture| capture.node.byte_range());

            ranges.collect()
        })
    }
}

impl LanguageScopedViewBuildStep for Python {
    fn lang() -> TSLanguage {
        tree_sitter_python::language()
    }

    fn parser() -> TSParser {
        let mut parser = TSParser::new();
        parser
            .set_language(Self::lang())
            .expect("Error loading Python grammar");

        parser
    }
}
