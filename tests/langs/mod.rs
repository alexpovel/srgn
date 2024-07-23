mod csharp;
mod go;
mod hcl;
mod rust;
mod typescript;

use rstest::rstest;
use serde::{Deserialize, Serialize};
use srgn::scoping::{
    langs::{
        python::{PreparedPythonQuery, Python},
        CodeQuery, LanguageScoper,
    },
    regex::Regex,
    scope::Scope,
    view::ScopedViewBuilder,
};
use std::{fs::read_to_string, ops::Range, path::Path};

fn get_input_output(lang: &str, file: &str) -> (String, String) {
    let path = Path::new("tests/langs");
    let path = path.join(lang);

    let input = read_to_string(path.join(format!("in/{file}"))).unwrap();
    let output = read_to_string(path.join(format!("out/{file}"))).unwrap();

    (input, output)
}

/// Nuke the target character from the input.
///
/// Convenience function for testing, as deleting a specific character, while
/// *retaining* it elsewhere, where the language did *not* scope down, is an easy way to
/// test.
fn nuke_target(input: &str, lang: &impl LanguageScoper) -> String {
    let mut builder = ScopedViewBuilder::new(input);

    builder.explode(lang);

    // Needs to be ASCII such that we can target e.g. variable names.
    let target = String::from("__T__");
    builder.explode(&Regex::try_from(target).unwrap());

    let mut view = builder.build();
    view.delete();

    view.to_string()
}

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
    fn new(line_number: usize, line_contents: String, match_: String, span: Range<usize>) -> Self {
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
            line_contents[span], match_,
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
                    let part =
                        InScopeLinePart::new(i + 1, line.to_string(), contents.to_string(), span);
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
