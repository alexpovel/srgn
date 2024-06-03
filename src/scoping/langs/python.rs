use super::{CodeQuery, Language, LanguageScoper, TSLanguage, TSQuery};
use crate::scoping::{langs::IGNORE, ROScopes, Scoper};
use clap::ValueEnum;
use const_format::concatcp;
use std::{fmt::Debug, str::FromStr};
use tree_sitter::QueryError;

/// The Python language.
pub type Python = Language<PythonQuery>;
/// A query for Python.
pub type PythonQuery = CodeQuery<CustomPythonQuery, PremadePythonQuery>;

/// Premade tree-sitter queries for Python.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PremadePythonQuery {
    /// Comments.
    Comments,
    /// Strings (raw, byte, f-strings; interpolation is respected; quotes included).
    Strings,
    /// Module names in imports (incl. periods; excl. `import`/`from`/`as`/`*`).
    Imports,
    /// Docstrings (not including multi-line strings).
    DocStrings,
    /// Function names, at the definition site.
    FunctionNames,
    /// Function calls.
    FunctionCalls,
}

impl From<PremadePythonQuery> for TSQuery {
    fn from(value: PremadePythonQuery) -> Self {
        TSQuery::new(
            Python::lang(),
            match value {
                PremadePythonQuery::Comments => "(comment) @comment",
                PremadePythonQuery::Strings => {
                    // Match either normal `string`s or `string`s with `interpolation`;
                    // using only the latter doesn't include the former.
                    concatcp!(
                        "
                    [
                        (string)
                        (string (interpolation) @",
                        IGNORE,
                        ")
                    ]
                    @string"
                    )
                }
                PremadePythonQuery::Imports => {
                    r"[
                        (import_statement
                                name: (dotted_name) @dn)
                        (import_from_statement
                                module_name: (dotted_name) @dn)
                        (import_from_statement
                                module_name: (dotted_name) @dn
                                    (wildcard_import))
                        (import_statement(
                            aliased_import
                                name: (dotted_name) @dn))
                        (import_from_statement
                            module_name: (relative_import) @ri)
                    ]"
                }
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

/// A custom tree-sitter query for Python.
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

impl Scoper for Python {
    fn scope<'viewee>(&self, input: &'viewee str) -> ROScopes<'viewee> {
        ROScopes::from_raw_ranges(
            input,
            Self::scope_via_query(&mut self.query(), input).into(),
        )
    }
}

impl LanguageScoper for Python {
    fn lang() -> TSLanguage {
        tree_sitter_python::language()
    }

    fn query(&self) -> TSQuery {
        self.query.clone().into()
    }
}
