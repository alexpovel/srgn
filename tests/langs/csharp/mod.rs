use rstest::rstest;
use srgn::scoping::langs::csharp::{CSharp, CSharpQuery, PremadeCSharpQuery};

use super::{get_input_output, nuke_target};

#[rstest]
#[case("comments.cs", CSharpQuery::Premade(PremadeCSharpQuery::Comments))]
#[case("strings.cs", CSharpQuery::Premade(PremadeCSharpQuery::Strings))]
fn test_csharp(#[case] file: &str, #[case] query: CSharpQuery) {
    let lang = CSharp::new(query);

    let (input, output) = get_input_output("csharp", file);
    let result = nuke_target(&input, &lang);

    assert_eq!(result, output);
}
