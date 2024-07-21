use super::{get_input_output, nuke_target};
use pretty_assertions::assert_eq;
use rstest::rstest;
use srgn::scoping::langs::python::{PreparedPythonQuery, Python, PythonQuery};

#[rstest]
#[case("docstring.py", PythonQuery::Prepared(PreparedPythonQuery::DocStrings))]
#[case("strings.py", PythonQuery::Prepared(PreparedPythonQuery::Strings))]
#[case("imports.py", PythonQuery::Prepared(PreparedPythonQuery::Imports))]
#[case("comments-lf.py", PythonQuery::Prepared(PreparedPythonQuery::Comments))]
#[case(
    "comments-crlf.py",
    PythonQuery::Prepared(PreparedPythonQuery::Comments)
)]
#[case(
    "function-names.py",
    PythonQuery::Prepared(PreparedPythonQuery::FunctionNames)
)]
#[case(
    "function-calls.py",
    PythonQuery::Prepared(PreparedPythonQuery::FunctionCalls)
)]
#[case("class.py", PythonQuery::Prepared(PreparedPythonQuery::Class))]
fn test_python_nuke(#[case] file: &str, #[case] query: PythonQuery) {
    let lang = Python::new(query);

    let (input, output) = get_input_output("python", file);
    let result = nuke_target(&input, &lang);

    assert_eq!(result, output);
}
