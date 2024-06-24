use super::{get_input_output, nuke_target};
use pretty_assertions::assert_eq;
use rstest::rstest;
use srgn::scoping::langs::hcl::{Hcl, HclQuery, PreparedHclQuery};

#[rstest]
#[case("variables.tf", HclQuery::Prepared(PreparedHclQuery::Variables))]
#[case(
    "resource-types.tf",
    HclQuery::Prepared(PreparedHclQuery::ResourceTypes)
)]
#[case(
    "resource-names.tf",
    HclQuery::Prepared(PreparedHclQuery::ResourceNames)
)]
#[case("data-names.tf", HclQuery::Prepared(PreparedHclQuery::DataNames))]
#[case("data-sources.tf", HclQuery::Prepared(PreparedHclQuery::DataSources))]
#[case("comments.tf", HclQuery::Prepared(PreparedHclQuery::Comments))]
#[case("strings.tf", HclQuery::Prepared(PreparedHclQuery::Strings))]
fn test_hcl_nuke(#[case] file: &str, #[case] query: HclQuery) {
    let lang = Hcl::new(query);

    let (input, output) = get_input_output("hcl", file);
    let result = nuke_target(&input, &lang);

    assert_eq!(result, output);
}
