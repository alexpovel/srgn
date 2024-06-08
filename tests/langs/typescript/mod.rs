use super::{get_input_output, nuke_target};
use pretty_assertions::assert_eq;
use rstest::rstest;
use srgn::scoping::langs::typescript::{PremadeTypeScriptQuery, TypeScript, TypeScriptQuery};

#[rstest]
#[case(
    "comments.ts",
    TypeScriptQuery::Premade(PremadeTypeScriptQuery::Comments)
)]
#[case(
    "strings.ts",
    TypeScriptQuery::Premade(PremadeTypeScriptQuery::Strings)
)]
#[case(
    "imports.ts",
    TypeScriptQuery::Premade(PremadeTypeScriptQuery::Imports)
)]
fn test_typescript_nuke(#[case] file: &str, #[case] query: TypeScriptQuery) {
    let lang = TypeScript::new(query);

    let (input, output) = get_input_output("typescript", file);
    let result = nuke_target(&input, &lang);

    assert_eq!(result, output);
}
