use std::fmt::Debug;

use clap::ValueEnum;
use const_format::formatcp;

use super::{CodeQuery, Find, Kind, Language, LanguageScoper, TSLanguage, TSQuery};
use crate::scoping::langs::IGNORE;

/// A type used to make the generic `Language` struct specific to the Python language and
/// provides the appropriate `tree_sitter::Language` object.
#[derive(Clone, Copy, Debug)]
pub struct LangKind {}

impl Kind for LangKind {
    fn ts_lang() -> TSLanguage {
        tree_sitter_python::LANGUAGE.into()
    }
}

/// The Python language.
pub type Python = Language<LangKind>;

/// Prepared tree-sitter queries for Python.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PreparedPythonQuery {
    /// Comments.
    Comments,
    /// Strings (raw, byte, f-strings; interpolation not included).
    Strings,
    /// Module names in imports (incl. periods; excl. `import`/`from`/`as`/`*`).
    Imports,
    /// Docstrings (not including multi-line strings).
    DocStrings,
    /// Function names, at the definition site.
    FunctionNames,
    /// Function calls.
    FunctionCalls,
    /// Class definitions (in their entirety).
    Class,
    /// Function definitions (*all* `def` block in their entirety).
    Def,
    /// Async function definitions (*all* `async def` block in their entirety).
    AsyncDef,
    /// Function definitions inside `class` bodies.
    Methods,
    /// Function definitions decorated as `classmethod` (excl. the decorator).
    ClassMethods,
    /// Function definitions decorated as `staticmethod` (excl. the decorator).
    StaticMethods,
    /// `with` blocks (in their entirety).
    With,
    /// `try` blocks (in their entirety).
    Try,
    /// `lambda` statements (in their entirety).
    Lambda,
    /// Global, i.e. module-level variables.
    Globals,
    /// Identifiers for variables (left-hand side of assignments).
    VariableIdentifiers,
    /// Types in type hints.
    Types,
    /// Identifiers (variable names, ...).
    Identifiers,
}

impl From<PreparedPythonQuery> for CodeQuery<'static> {
    #[allow(clippy::too_many_lines)]
    fn from(value: PreparedPythonQuery) -> Self {
        let s = match value {
            PreparedPythonQuery::Comments => "(comment) @comment",
            PreparedPythonQuery::Strings => "(string_content) @string",
            PreparedPythonQuery::Imports => {
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
            PreparedPythonQuery::DocStrings => {
                // Triple-quotes are also used for multi-line strings. So look only
                // for stand-alone expressions, which are not part of some variable
                // assignment.
                formatcp!(
                    "
                    (
                        (expression_statement
                            (string
                                (string_start) @{0}
                                (string_content) @string
                                (#match? @{0} \"\\^\\\"\\\"\\\"\")
                            )
                        )
                    )
                    ",
                    IGNORE
                )
            }
            PreparedPythonQuery::FunctionNames => {
                r"
                (function_definition
                    name: (identifier) @function-name
                )
                "
            }
            PreparedPythonQuery::FunctionCalls => {
                r"
                (call
                    function: (identifier) @function-name
                )
                "
            }
            PreparedPythonQuery::Class => "(class_definition) @class",
            PreparedPythonQuery::Def => "(function_definition) @def",
            PreparedPythonQuery::AsyncDef => {
                r#"((function_definition) @def (#match? @def "^async "))"#
            }
            PreparedPythonQuery::Methods => {
                r"
                (class_definition
                    body: (block
                        [
                            (function_definition) @method
                            (decorated_definition definition: (function_definition)) @method
                        ]
                    )
                )
                "
            }
            PreparedPythonQuery::ClassMethods => {
                formatcp!(
                    "
                    (class_definition
                        body: (block
                            (decorated_definition
                                (decorator (identifier) @{0})
                                definition: (function_definition) @method
                                (#eq? @{0} \"classmethod\")
                            )
                        )
                    )",
                    IGNORE
                )
            }
            PreparedPythonQuery::StaticMethods => {
                formatcp!(
                    "
                    (class_definition
                        body: (block
                            (decorated_definition
                                (decorator (identifier) @{0})
                                definition: (function_definition) @method
                                (#eq? @{0} \"staticmethod\")
                            )
                        )
                    )",
                    IGNORE
                )
            }
            PreparedPythonQuery::With => "(with_statement) @with",
            PreparedPythonQuery::Try => "(try_statement) @try",
            PreparedPythonQuery::Lambda => "(lambda) @lambda",
            PreparedPythonQuery::Globals => {
                "(module (expression_statement (assignment left: (identifier) @global)))"
            }
            PreparedPythonQuery::VariableIdentifiers => {
                "(assignment left: (identifier) @identifier)"
            }
            PreparedPythonQuery::Types => "(type) @type",
            PreparedPythonQuery::Identifiers => "(identifier) @identifier",
        };

        s.into()
    }
}

impl LanguageScoper for Python {
    fn lang() -> TSLanguage {
        tree_sitter_python::LANGUAGE.into()
    }

    fn pos_query(&self) -> &TSQuery {
        &self.positive_query
    }

    fn neg_query(&self) -> Option<&TSQuery> {
        self.negative_query.as_ref()
    }
}

impl Find for Python {
    fn extensions(&self) -> &'static [&'static str] {
        &["py"]
    }

    fn interpreters(&self) -> Option<&'static [&'static str]> {
        Some(&["python", "python3"])
    }
}
