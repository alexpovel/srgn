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
    FunctionNames,
    FunctionCalls,
}

impl From<PremadePythonQuery> for TSQuery {
    fn from(value: PremadePythonQuery) -> Self {
        TSQuery::new(
            Python::lang(),
            match value {
                PremadePythonQuery::Comments => "(comment) @comment",
                PremadePythonQuery::DocStrings => {
                    // Triple-quotes are also used for multi-line strings. So look only
                    // for stand-alone expressions, which are not part of some variable
                    // assignment.
                    r#"
                    (
                        (expression_statement
                            (string) @string
                            (#match? @string "^\"\"\"")
                        )
                    )
                    "#
                }
                PremadePythonQuery::FunctionNames => {
                    r"
                    (function_definition
                        name: (identifier) @function-name
                    )
                    "
                }
                PremadePythonQuery::FunctionCalls => {
                    r"
                    (call
                        function: (identifier) @function-name
                    )
                    "
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
    fn scope<'viewee>(&self, input: &'viewee str) -> ScopedViewBuilder<'viewee> {
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
