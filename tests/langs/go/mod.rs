use super::{get_input_output, nuke_target};
use pretty_assertions::assert_eq;
use rstest::rstest;
use srgn::scoping::langs::go::{Go, GoQuery, PreparedGoQuery};

#[rstest]
#[case("comments.go", GoQuery::Prepared(PreparedGoQuery::Comments))]
#[case("strings.go", GoQuery::Prepared(PreparedGoQuery::Strings))]
#[case("imports.go", GoQuery::Prepared(PreparedGoQuery::Imports))]
#[case("struct-tags.go", GoQuery::Prepared(PreparedGoQuery::StructTags))]
fn test_go_nuke(#[case] file: &str, #[case] query: GoQuery) {
    let lang = Go::new(query);

    let (input, output) = get_input_output("go", file);
    let result = nuke_target(&input, &lang);

    assert_eq!(result, output);
}
