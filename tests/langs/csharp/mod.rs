use super::{get_input_output, nuke_target};
use pretty_assertions::assert_eq;
use rstest::rstest;
use srgn::scoping::langs::csharp::{CSharp, CSharpQuery, PreparedCSharpQuery};

#[rstest]
#[case("comments.cs", CSharpQuery::Prepared(PreparedCSharpQuery::Comments))]
#[case("strings.cs", CSharpQuery::Prepared(PreparedCSharpQuery::Strings))]
#[case("usings.cs", CSharpQuery::Prepared(PreparedCSharpQuery::Usings))]
fn test_csharp_nuke(#[case] file: &str, #[case] query: CSharpQuery) {
    let lang = CSharp::new(query);

    let (input, output) = get_input_output("csharp", file);
    let result = nuke_target(&input, &lang);

    assert_eq!(result, output);
}
