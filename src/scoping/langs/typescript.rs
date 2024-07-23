use super::{CodeQuery, Find, Language, LanguageScoper, TSLanguage, TSQuery};
use crate::scoping::{scope::RangesWithContext, Scoper};
use clap::ValueEnum;
use std::{fmt::Debug, str::FromStr};
use tree_sitter::QueryError;

/// The TypeScript language.
pub type TypeScript = Language<TypeScriptQuery>;
/// A query for TypeScript.
pub type TypeScriptQuery = CodeQuery<CustomTypeScriptQuery, PreparedTypeScriptQuery>;
/// Prepared tree-sitter queries for TypeScript.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PreparedTypeScriptQuery {
    /// Comments.
    Comments,
    /// Strings (literal, template).
    Strings,
    /// Imports (module specifiers).
    Imports,
}

impl From<PreparedTypeScriptQuery> for TSQuery {
    fn from(value: PreparedTypeScriptQuery) -> Self {
        TSQuery::new(
            &TypeScript::lang(),
            match value {
                PreparedTypeScriptQuery::Comments => "(comment) @comment",
                PreparedTypeScriptQuery::Imports => {
                    r"(import_statement source: (string (string_fragment) @sf))"
                }
                PreparedTypeScriptQuery::Strings => "(string_fragment) @string",
            },
        )
        .expect("Prepared queries to be valid")
    }
}

/// A custom tree-sitter query for TypeScript.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomTypeScriptQuery(String);

impl FromStr for CustomTypeScriptQuery {
    type Err = QueryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TSQuery::new(&TypeScript::lang(), s) {
            Ok(_) => Ok(Self(s.to_string())),
            Err(e) => Err(e),
        }
    }
}

impl From<CustomTypeScriptQuery> for TSQuery {
    fn from(value: CustomTypeScriptQuery) -> Self {
        TSQuery::new(&TypeScript::lang(), &value.0)
            .expect("Valid query, as object cannot be constructed otherwise")
    }
}

impl Scoper for TypeScript {
    fn scope_raw<'viewee>(&self, input: &'viewee str) -> RangesWithContext<'viewee> {
        Self::scope_via_query(&mut self.query(), input).into()
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

impl Find for TypeScript {
    fn extensions(&self) -> &'static [&'static str] {
        &["ts", "tsx"]
    }
}
