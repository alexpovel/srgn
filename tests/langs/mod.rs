use rstest::rstest;
use serde::{Deserialize, Serialize};
use srgn::scoping::{
    langs::{
        csharp::{CSharp, PreparedCSharpQuery},
        go::{Go, PreparedGoQuery},
        hcl::{Hcl, PreparedHclQuery},
        python::{PreparedPythonQuery, Python},
        rust::{PreparedRustQuery, Rust},
        typescript::{PreparedTypeScriptQuery, TypeScript},
        CodeQuery, LanguageScoper,
    },
    scope::Scope,
    view::ScopedViewBuilder,
};
use std::ops::Range;

/// A type that when serialized, will visually highlight the portions of a line which
/// were matched.
///
/// Single letter names to not disrupt alignment.
///
/// For example, when serialized as YAML, produces output such as
///
/// ```yaml
/// n: 156
/// l: "        print(f\"Loop iteration {i}\")\n"
/// m: "        ^^^^^                           "
/// ```
///
/// with `m` indicating the [`Scope::In`] part of the `l`ine.
#[derive(Debug, Serialize, Deserialize)]
struct InScopeLinePart {
    /// Line number.
    n: usize,
    /// Line itself, in original form.
    l: String,
    /// The string highlighting the matching. Has to be serialized *below*.
    m: String,
}

impl InScopeLinePart {
    fn new(line_number: usize, line_contents: String, match_: &str, span: Range<usize>) -> Self {
        // Split into the three components
        let (start, mid, end) = (
            &line_contents[..span.start],
            &line_contents[span.clone()],
            &line_contents[span.end..],
        );

        // ASSUMPTION is that `.escape_default()` will escape the same way (YAML, ...)
        // serialization does. Important for alignment to be correct.

        // Leading space
        let mut m = " ".repeat(start.escape_default().to_string().len());
        // The highlights for the line above.
        m.push_str(&"^".repeat(mid.escape_default().to_string().len()));
        // Trailing spaces; not strictly necessary but slightly nicer.
        m.push_str(&" ".repeat(end.escape_default().to_string().len()));

        assert_eq!(
            &line_contents[span], match_,
            "What is highlighted is what was matched"
        );

        Self {
            n: line_number,
            l: line_contents,
            m,
        }
    }
}

#[rstest]
#[case(
    "base.py_comments",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::Comments)),
)]
#[case(
    "base.py_strings",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::Strings)),
)]
#[case(
    "base.py_imports",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::Imports)),
)]
#[case(
    "base.py_docstrings",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::DocStrings)),
)]
#[case(
    "base.py_function-names",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::FunctionNames)),
)]
#[case(
    "base.py_function-calls",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::FunctionCalls)),
)]
#[case(
    "base.py_class",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::Class)),
)]
#[case(
    "base.py_def",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::Def)),
)]
#[case(
    "base.py_async-def",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::AsyncDef)),
)]
#[case(
    "base.py_methods",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::Methods)),
)]
#[case(
    "base.py_classmethods",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::ClassMethods)),
)]
#[case(
    "base.py_staticmethods",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::StaticMethods)),
)]
#[case(
    "base.py_with",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::With)),
)]
#[case(
    "base.py_try",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::Try)),
)]
#[case(
    "base.py_lambda",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::Lambda)),
)]
#[case(
    "base.py_globals",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::Globals)),
)]
#[case(
    "base.py_identifiers",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::VariableIdentifiers)),
)]
#[case(
    "base.py_types",
    include_str!("python/base.py"),
    Python::new(CodeQuery::Prepared(PreparedPythonQuery::Types)),
)]
#[case(
    "base.ts_strings",
    include_str!("typescript/base.ts"),
    TypeScript::new(CodeQuery::Prepared(PreparedTypeScriptQuery::Strings)),
)]
#[case(
    "base.ts_comments",
    include_str!("typescript/base.ts"),
    TypeScript::new(CodeQuery::Prepared(PreparedTypeScriptQuery::Comments)),
)]
#[case(
    "base.ts_imports",
    include_str!("typescript/base.ts"),
    TypeScript::new(CodeQuery::Prepared(PreparedTypeScriptQuery::Imports)),
)]
#[case(
    "base.rs_strings",
    include_str!("rust/base.rs"),
    Rust::new(CodeQuery::Prepared(PreparedRustQuery::Strings)),
)]
#[case(
    "base.rs_comments",
    include_str!("rust/base.rs"),
    Rust::new(CodeQuery::Prepared(PreparedRustQuery::Comments)),
)]
#[case(
    "base.rs_uses",
    include_str!("rust/base.rs"),
    Rust::new(CodeQuery::Prepared(PreparedRustQuery::Uses)),
)]
#[case(
    "base.rs_doc-comments",
    include_str!("rust/base.rs"),
    Rust::new(CodeQuery::Prepared(PreparedRustQuery::DocComments)),
)]
#[case(
    "base.tf_variables",
    include_str!("hcl/base.tf"),
    Hcl::new(CodeQuery::Prepared(PreparedHclQuery::Variables)),
)]
#[case(
    "base.tf_resource-types",
    include_str!("hcl/base.tf"),
    Hcl::new(CodeQuery::Prepared(PreparedHclQuery::ResourceTypes)),
)]
#[case(
    "base.tf_resource-names",
    include_str!("hcl/base.tf"),
    Hcl::new(CodeQuery::Prepared(PreparedHclQuery::ResourceNames)),
)]
#[case(
    "base.tf_data-names",
    include_str!("hcl/base.tf"),
    Hcl::new(CodeQuery::Prepared(PreparedHclQuery::DataNames)),
)]
#[case(
    "base.tf_data-sources",
    include_str!("hcl/base.tf"),
    Hcl::new(CodeQuery::Prepared(PreparedHclQuery::DataSources)),
)]
#[case(
    "base.tf_comments",
    include_str!("hcl/base.tf"),
    Hcl::new(CodeQuery::Prepared(PreparedHclQuery::Comments)),
)]
#[case(
    "base.tf_strings",
    include_str!("hcl/base.tf"),
    Hcl::new(CodeQuery::Prepared(PreparedHclQuery::Strings)),
)]
#[case(
    "base.go_comments",
    include_str!("go/base.go"),
    Go::new(CodeQuery::Prepared(PreparedGoQuery::Comments)),
)]
#[case(
    "base.go_strings",
    include_str!("go/base.go"),
    Go::new(CodeQuery::Prepared(PreparedGoQuery::Strings)),
)]
#[case(
    "base.go_imports",
    include_str!("go/base.go"),
    Go::new(CodeQuery::Prepared(PreparedGoQuery::Imports)),
)]
#[case(
    "base.go_struct-tags",
    include_str!("go/base.go"),
    Go::new(CodeQuery::Prepared(PreparedGoQuery::StructTags)),
)]
#[case(
    "base.cs_strings",
    include_str!("csharp/base.cs"),
    CSharp::new(CodeQuery::Prepared(PreparedCSharpQuery::Strings)),
)]
#[case(
    "base.cs_usings",
    include_str!("csharp/base.cs"),
    CSharp::new(CodeQuery::Prepared(PreparedCSharpQuery::Usings)),
)]
#[case(
    "base.cs_comments",
    include_str!("csharp/base.cs"),
    CSharp::new(CodeQuery::Prepared(PreparedCSharpQuery::Comments)),
)]
fn test_language_scopers(
    #[case] snapshot_name: &str,
    #[case] contents: &str,
    #[case] lang: impl LanguageScoper,
) {
    let mut builder = ScopedViewBuilder::new(contents);
    builder.explode(&lang);
    let view = builder.build();

    // Collect only those lines which are in scope. This avoids enormous clutter from
    // out of scope items. The in-scope lines are *pinned* by lines numbers and
    // highlights/their span, so false-positives and confusion around multiple matches
    // per line are avoided.
    let inscope_parts: Vec<InScopeLinePart> = view
        .lines()
        .into_iter()
        .enumerate()
        .flat_map(|(i, line)| {
            let mut start = 0;

            line.scopes()
                .0
                .clone()
                .into_iter()
                .filter_map(move |scope| {
                    let contents: &str = (&scope).into();
                    let end = start + contents.len();
                    let span = start..end;
                    let part = InScopeLinePart::new(i + 1, line.to_string(), contents, span);
                    start = end;

                    match scope.0 {
                        Scope::In(..) => Some(part),
                        Scope::Out(..) => None,
                    }
                })
        })
        .collect();

    insta::assert_yaml_snapshot!(snapshot_name, inscope_parts);
}
