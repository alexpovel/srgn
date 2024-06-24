use super::{get_input_output, nuke_target};
use pretty_assertions::assert_eq;
use rstest::rstest;
use srgn::scoping::langs::typescript::{PreparedTypeScriptQuery, TypeScript, TypeScriptQuery};

#[rstest]
#[case(
    "comments.ts",
    TypeScriptQuery::Prepared(PreparedTypeScriptQuery::Comments)
)]
#[case(
    "strings.ts",
    TypeScriptQuery::Prepared(PreparedTypeScriptQuery::Strings)
)]
#[case(
    "imports.ts",
    TypeScriptQuery::Prepared(PreparedTypeScriptQuery::Imports)
)]
fn test_typescript_nuke(#[case] file: &str, #[case] query: TypeScriptQuery) {
    let lang = TypeScript::new(query);

    let (input, output) = get_input_output("typescript", file);
    let result = nuke_target(&input, &lang);

    assert_eq!(result, output);
}
