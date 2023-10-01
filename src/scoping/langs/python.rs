use super::{CodeQuery, Language, LanguageScopedViewBuildStep, TSLanguage, TSQuery};
use crate::scoping::{ScopedViewBuildStep, ScopedViewBuilder};
use clap::ValueEnum;
use std::{fmt::Debug, str::FromStr};
use tree_sitter::QueryError;

pub type Python = Language<PythonQuery>;
pub type PythonQuery = CodeQuery<CustomPythonQuery, PremadePythonQuery>;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PremadePythonQuery {
    Comments,
    DocStrings,
}

impl From<PremadePythonQuery> for TSQuery {
    fn from(value: PremadePythonQuery) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomPythonQuery(String);

impl FromStr for CustomPythonQuery {
    type Err = QueryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TSQuery::new(Python::lang(), s) {
            Ok(_) => Ok(Self(s.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<CustomPythonQuery> for TSQuery {
    fn from(value: CustomPythonQuery) -> Self {
        TSQuery::new(Python::lang(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl ScopedViewBuildStep for Python {
    fn scope<'a>(&self, input: &'a str) -> ScopedViewBuilder<'a> {
        self.scope_via_query(input)
    }
}

impl LanguageScopedViewBuildStep for Python {
    fn lang() -> TSLanguage {
        tree_sitter_python::language()
    }

    fn query(&self) -> TSQuery {
        self.query.clone().into()
    }
}
