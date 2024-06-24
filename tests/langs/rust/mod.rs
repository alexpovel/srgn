use super::{get_input_output, nuke_target};
use pretty_assertions::assert_eq;
use rstest::rstest;
use srgn::scoping::langs::rust::{PreparedRustQuery, Rust, RustQuery};

#[rstest]
#[case("comments.rs", RustQuery::Prepared(PreparedRustQuery::Comments))]
#[case("doc-comments.rs", RustQuery::Prepared(PreparedRustQuery::DocComments))]
#[case("uses.rs", RustQuery::Prepared(PreparedRustQuery::Uses))]
#[case("strings.rs", RustQuery::Prepared(PreparedRustQuery::Strings))]
fn test_rust_nuke(#[case] file: &str, #[case] query: RustQuery) {
    let lang = Rust::new(query);

    let (input, output) = get_input_output("rust", file);
    let result = nuke_target(&input, &lang);

    assert_eq!(result, output);
}
