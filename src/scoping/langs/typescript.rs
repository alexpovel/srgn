use super::{CodeQuery, Language, LanguageScopedViewBuildStep, TSLanguage, TSQuery};
use crate::scoping::{ScopedViewBuildStep, ScopedViewBuilder};
use clap::ValueEnum;
use std::{fmt::Debug, str::FromStr};
use tree_sitter::QueryError;

pub type TypeScript = Language<TypeScriptQuery>;
pub type TypeScriptQuery = CodeQuery<CustomTypeScriptQuery, PremadeTypeScriptQuery>;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PremadeTypeScriptQuery {
    Comments,
}

impl From<PremadeTypeScriptQuery> for TSQuery {
    fn from(value: PremadeTypeScriptQuery) -> Self {
        TSQuery::new(
            TypeScript::lang(),
            match value {
                PremadeTypeScriptQuery::Comments => "(comment) @comment",
            },
        )
        .expect("Premade queries to be valid")
    }
}

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

impl ScopedViewBuildStep for TypeScript {
    fn scope<'viewee>(&self, input: &'viewee str) -> ScopedViewBuilder<'viewee> {
        self.scope_via_query(input)
    }
}

impl LanguageScopedViewBuildStep for TypeScript {
    fn lang() -> TSLanguage {
        tree_sitter_typescript::language_typescript()
    }

    fn query(&self) -> TSQuery {
        self.query.clone().into()
    }
}
