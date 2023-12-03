use rstest::rstest;
use srgn::scoping::langs::rust::{PremadeRustQuery, Rust, RustQuery};

use super::{get_input_output, nuke_target};

#[rstest]
#[case("comments.rs", RustQuery::Premade(PremadeRustQuery::Comments))]
#[case("doc-comments.rs", RustQuery::Premade(PremadeRustQuery::DocComments))]
#[case("strings.rs", RustQuery::Premade(PremadeRustQuery::Strings))]
fn test_rust_nuke(#[case] file: &str, #[case] query: RustQuery) {
    let lang = Rust::new(query);

    let (input, output) = get_input_output("rust", file);
    let result = nuke_target(&input, &lang);

    assert_eq!(result, output);
}
