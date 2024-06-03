use super::{CodeQuery, Language, LanguageScoper, TSLanguage, TSQuery};
use crate::scoping::{langs::IGNORE, ROScopes, Scoper};
use clap::ValueEnum;
use const_format::concatcp;
use std::{fmt::Debug, str::FromStr};
use tree_sitter::QueryError;

/// The TypeScript language.
pub type TypeScript = Language<TypeScriptQuery>;
/// A query for TypeScript.
pub type TypeScriptQuery = CodeQuery<CustomTypeScriptQuery, PremadeTypeScriptQuery>;
/// Premade tree-sitter queries for TypeScript.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PremadeTypeScriptQuery {
    /// Comments.
    Comments,
    /// Strings (literal, template; includes quote characters).
    Strings,
    /// Imports (module specifiers).
    Imports,
}

impl From<PremadeTypeScriptQuery> for TSQuery {
    fn from(value: PremadeTypeScriptQuery) -> Self {
        TSQuery::new(
            TypeScript::lang(),
            match value {
                PremadeTypeScriptQuery::Comments => "(comment) @comment",
                PremadeTypeScriptQuery::Imports => {
                    r"(import_statement source: (string (string_fragment) @sf))"
                }
                PremadeTypeScriptQuery::Strings => {
                    concatcp!(
                        "
                    [
                        (string)
                        (template_string (template_substitution) @",
                        IGNORE,
                        ")
                    ]
                    @string"
                    )
                }
            },
        )
        .expect("Premade queries to be valid")
    }
}

/// A custom tree-sitter query for TypeScript.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomTypeScriptQuery(String);

impl FromStr for CustomTypeScriptQuery {
    type Err = QueryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TSQuery::new(TypeScript::lang(), s) {
            Ok(_) => Ok(Self(s.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<CustomTypeScriptQuery> for TSQuery {
    fn from(value: CustomTypeScriptQuery) -> Self {
        TSQuery::new(TypeScript::lang(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl Scoper for TypeScript {
    fn scope<'viewee>(&self, input: &'viewee str) -> ROScopes<'viewee> {
        ROScopes::from_raw_ranges(
            input,
            Self::scope_via_query(&mut self.query(), input).into(),
        )
    }
}

impl LanguageScoper for TypeScript {
    fn lang() -> TSLanguage {
        tree_sitter_typescript::language_typescript()
    }

    fn query(&self) -> TSQuery {
        self.query.clone().into()
    }
}
