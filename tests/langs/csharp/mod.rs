use rstest::rstest;
use srgn::scoping::{
    langs::csharp::{CSharp, CSharpQuery, PremadeCSharpQuery},
    view::ScopedViewBuilder,
};

use super::get_input_output;

#[rstest]
#[case("comments.cs", CSharpQuery::Premade(PremadeCSharpQuery::Comments))]
fn test_csharp(#[case] file: &str, #[case] query: CSharpQuery) {
    let lang = CSharp::new(query);

    let (input, output) = get_input_output("csharp", file);

    let mut builder = ScopedViewBuilder::new(&input);
    builder.explode(&lang);
    let mut view = builder.build();
    view.delete();

    assert_eq!(view.to_string(), output);
}
