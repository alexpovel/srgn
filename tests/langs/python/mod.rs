use rstest::rstest;
use srgn::scoping::{
    langs::python::{PremadePythonQuery, Python, PythonQuery},
    view::ScopedViewBuilder,
};

use super::get_input_output;

#[rstest]
#[case("docstring.py", PythonQuery::Premade(PremadePythonQuery::DocStrings))]
#[case("comments-lf.py", PythonQuery::Premade(PremadePythonQuery::Comments))]
#[case("comments-crlf.py", PythonQuery::Premade(PremadePythonQuery::Comments))]
#[case(
    "function-names.py",
    PythonQuery::Premade(PremadePythonQuery::FunctionNames)
)]
#[case(
    "function-calls.py",
    PythonQuery::Premade(PremadePythonQuery::FunctionCalls)
)]
fn test_python(#[case] file: &str, #[case] query: PythonQuery) {
    let lang = Python::new(query);

    let (input, output) = get_input_output("python", file);

    let mut builder = ScopedViewBuilder::new(&input);
    builder.explode(&lang);
    let mut view = builder.build();
    view.delete();

    assert_eq!(view.to_string(), output);
}