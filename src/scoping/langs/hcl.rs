use std::fmt::Debug;

use clap::ValueEnum;

use super::{LanguageScoper, QuerySource, TSLanguage, TSQuery, TSQueryError, TreeSitterRegex};
use crate::find::Find;
use crate::scoping::langs::IGNORE;

/// A compiled query for the Hashicorp Configuration language.
#[derive(Debug)]
pub struct CompiledQuery(super::CompiledQuery);

impl TryFrom<QuerySource> for CompiledQuery {
    type Error = TSQueryError;

    /// Create a new compiled query for the Hashicorp Configuration language.
    ///
    /// # Errors
    ///
    /// See the concrete type of the [`TSQueryError`](tree_sitter::QueryError)variant for when this method errors.
    fn try_from(query: QuerySource) -> Result<Self, Self::Error> {
        let q = super::CompiledQuery::from_source(&tree_sitter_python::LANGUAGE.into(), &query)?;
        Ok(Self(q))
    }
}

impl From<PreparedQuery> for CompiledQuery {
    fn from(query: PreparedQuery) -> Self {
        Self(super::CompiledQuery::from_prepared_query(
            &tree_sitter_hcl::LANGUAGE.into(),
            &query.as_string(),
        ))
    }
}

/// Prepared tree-sitter queries for Hcl.
#[derive(Debug, Clone, ValueEnum)]
pub enum PreparedQuery {
    /// `variable` blocks (in their entirety).
    Variable,
    /// `resource` blocks (in their entirety).
    Resource,
    /// `data` blocks (in their entirety).
    Data,
    /// `output` blocks (in their entirety).
    Output,
    /// `provider` blocks (in their entirety).
    Provider,
    /// All `required_providers` from the `terraform` block.
    RequiredProviders,
    /// Entry from `required_providers` whose given name matches the pattern.
    #[value(skip)] // Non-unit enum variants need: https://github.com/clap-rs/clap/issues/2621
    RequiredProvidersNamed(TreeSitterRegex),
    /// `terraform` blocks (in their entirety).
    Terraform,
    /// `locals` blocks (in their entirety).
    Locals,
    /// `module` blocks (in their entirety).
    Module,
    /// Variable declarations and usages.
    Variables,
    /// `resource` name declarations and usages.
    ///
    /// In `resource "a" "b"`, only "b" is matched.
    ResourceNames,
    /// `resource` type declarations and usages.
    ///
    /// In `resource "a" "b"`, only "a" is matched.
    ResourceTypes,
    /// `data` name declarations and usages.
    ///
    /// In `data "a" "b"`, only "b" is matched.
    DataNames,
    /// `data` source declarations and usages.
    ///
    /// In `data "a" "b"`, only "a" is matched.
    DataSources,
    /// Comments.
    Comments,
    /// Literal strings.
    ///
    /// Excluding resource, variable, ... names as well as interpolation parts.
    Strings,
}

impl PreparedQuery {
    #[expect(clippy::too_many_lines)] // No good way to avoid
    fn as_string(&self) -> String {
        // Seems to not play nice with the macro. Put up here, else interpolation is
        // affected.
        match self {
            Self::Variable => r#"
                    (block
                        (identifier) @name
                        (#eq? @name "variable")
                    ) @block
                "#
            .into(),
            Self::Resource => r#"
                    (block
                        (identifier) @name
                        (#eq? @name "resource")
                    ) @block
                "#
            .into(),
            Self::Data => r#"
                    (block
                        (identifier) @name
                        (#eq? @name "data")
                    ) @block
                "#
            .into(),
            Self::Output => r#"
                    (block
                        (identifier) @name
                        (#eq? @name "output")
                    ) @block
                "#
            .into(),
            Self::Provider => r#"
                    (block
                        (identifier) @name
                        (#eq? @name "provider")
                    ) @block
                "#
            .into(),
            Self::RequiredProviders => {
                format!(
                    r#"
                    (block
                        (identifier) @{IGNORE}.level1.identifier
                        (body
                            (block
                                (identifier) @{IGNORE}.level2.identifier
                                (body) @body
                                (#eq? @{IGNORE}.level2.identifier "required_providers")
                            )
                        )
                        (#eq? @{IGNORE}.level1.identifier "terraform")
                    )
                    "#,
                )
            }
            Self::RequiredProvidersNamed(provider) => {
                format!(
                    r#"
                    (block
                        (identifier) @{IGNORE}.level1.identifier
                        (body
                            (block
                                (identifier) @{IGNORE}.level2.identifier
                                (body
                                    (attribute
                                        (identifier) @{IGNORE}.attribute.identifier
                                        (#match? @{IGNORE}.attribute.identifier "{provider}")
                                        (expression) @expr
                                    )
                                )
                                (#eq? @{IGNORE}.level2.identifier "required_providers")
                            )
                        )
                        (#eq? @{IGNORE}.level1.identifier "terraform")
                    )
                    "#,
                )
            }
            Self::Terraform => r#"
                    (block
                        (identifier) @name
                        (#eq? @name "terraform")
                    ) @block
                "#
            .into(),
            Self::Locals => r#"
                    (block
                        (identifier) @name
                        (#eq? @name "locals")
                    ) @block
                "#
            .into(),
            Self::Module => r#"
                    (block
                        (identifier) @name
                        (#eq? @name "module")
                    ) @block
                "#
            .into(),
            Self::Variables => {
                // Capturing nodes with names, such as `@id`, requires names to be
                // unique across the *entire* query, else things break. Hence, us
                // `@a.b` syntax (which seems undocumented).
                format!(
                    r#"
                        [
                            (block
                                (identifier) @{IGNORE}.declaration
                                (string_lit (template_literal) @name.declaration)
                                (#match? @{IGNORE}.declaration "variable")
                            )
                            (
                                (variable_expr
                                    (identifier) @{IGNORE}.usage
                                    (#match? @{IGNORE}.usage "var")
                                )
                                .
                                (get_attr
                                    (identifier) @name.usage
                                )
                            )
                        ]
                    "#
                )
            }
            Self::ResourceNames => {
                // Capturing nodes with names, such as `@id`, requires names to be
                // unique across the *entire* query, else things break. Hence, us
                // `@a.b` syntax (which seems undocumented).
                format!(
                    r#"
                        [
                            (block
                                (identifier) @{IGNORE}.declaration
                                (string_lit)
                                (string_lit (template_literal) @name.declaration)
                                (#match? @{IGNORE}.declaration "resource")
                            )
                            (
                                (variable_expr
                                    (identifier) @{IGNORE}.usage
                                    (#not-any-of? @{IGNORE}.usage
                                        "var"
                                        "data"
                                        "count"
                                        "module"
                                        "local"
                                    )
                                )
                                .
                                (get_attr
                                    (identifier) @name.usage
                                )
                            )
                        ]
                    "#
                )
            }
            Self::ResourceTypes => {
                // Capturing nodes with names, such as `@id`, requires names to be
                // unique across the *entire* query, else things break. Hence, us
                // `@a.b` syntax (which seems undocumented).
                format!(
                    r#"
                        [
                            (block
                                (identifier) @{IGNORE}.declaration
                                (string_lit (template_literal) @name.type)
                                (string_lit)
                                (#match? @{IGNORE}.declaration "resource")
                            )
                            (
                                (variable_expr
                                    .
                                    (identifier) @name.usage
                                    (#not-any-of? @name.usage
                                        "var"
                                        "data"
                                        "count"
                                        "module"
                                        "local"
                                    )
                                )
                                .
                                (get_attr
                                    (identifier)
                                )
                            )
                        ]
                    "#
                )
            }
            Self::DataNames => {
                // Capturing nodes with names, such as `@id`, requires names to be
                // unique across the *entire* query, else things break. Hence, us
                // `@a.b` syntax (which seems undocumented).
                format!(
                    r#"
                        [
                            (block
                                (identifier) @{IGNORE}.declaration
                                (string_lit)
                                (string_lit (template_literal) @name.declaration)
                                (#match? @{IGNORE}.declaration "data")
                            )
                            (
                                (variable_expr
                                    (identifier) @{IGNORE}.usage
                                    (#match? @{IGNORE}.usage "data")
                                )
                                .
                                (get_attr
                                    (identifier)
                                )
                                .
                                (get_attr
                                    (identifier) @name.usage
                                )
                            )
                        ]
                    "#
                )
            }
            Self::DataSources => {
                // Capturing nodes with names, such as `@id`, requires names to be
                // unique across the *entire* query, else things break. Hence, us
                // `@a.b` syntax (which seems undocumented).
                format!(
                    r#"
                        [
                            (block
                                (identifier) @{IGNORE}.declaration
                                (string_lit (template_literal) @name.provider)
                                (string_lit)
                                (#match? @{IGNORE}.declaration "data")
                            )
                            (
                                (variable_expr
                                    (identifier) @{IGNORE}.usage
                                    (#match? @{IGNORE}.usage "data")
                                )
                                .
                                (get_attr
                                    (identifier) @name.provider
                                )
                                .
                                (get_attr
                                    (identifier)
                                )
                            )
                        ]
                    "#
                )
            }
            Self::Comments => "(comment) @comment".into(),
            Self::Strings => r"
                [
                    (literal_value
                        (string_lit
                            (template_literal) @string.literal
                        )
                    )
                    (quoted_template
                        (template_literal) @string.template_literal
                    )
                    (heredoc_template
                        (template_literal) @string.heredoc_literal
                    )
                ]
                "
            .into(),
        }
    }
}

impl LanguageScoper for CompiledQuery {
    fn lang() -> TSLanguage {
        tree_sitter_hcl::LANGUAGE.into()
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
        &["hcl", "tf"]
    }
}
