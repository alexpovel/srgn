use rstest::rstest;
use srgn::scoping::langs::go::{Go, GoQuery, PremadeGoQuery};

use super::{get_input_output, nuke_target};

#[rstest]
#[case("comments.go", GoQuery::Premade(PremadeGoQuery::Comments))]
#[case("strings.go", GoQuery::Premade(PremadeGoQuery::Strings))]
#[case("struct-tags.go", GoQuery::Premade(PremadeGoQuery::StructTags))]
fn test_go_nuke(#[case] file: &str, #[case] query: GoQuery) {
    let lang = Go::new(query);

    let (input, output) = get_input_output("go", file);
    let result = nuke_target(&input, &lang);

    assert_eq!(result, output);
}
