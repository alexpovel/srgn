use rstest::rstest;
use srgn::scoping::langs::python::{PremadePythonQuery, Python, PythonQuery};

use super::{get_input_output, nuke_target};

#[rstest]
#[case("docstring.py", PythonQuery::Premade(PremadePythonQuery::DocStrings))]
#[case("strings.py", PythonQuery::Premade(PremadePythonQuery::Strings))]
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
fn test_python_nuke(#[case] file: &str, #[case] query: PythonQuery) {
    let lang = Python::new(query);

    let (input, output) = get_input_output("python", file);
    let result = nuke_target(&input, &lang);

    assert_eq!(result, output);
}
