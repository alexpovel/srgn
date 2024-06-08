use super::{get_input_output, nuke_target};
use pretty_assertions::assert_eq;
use rstest::rstest;
use srgn::scoping::langs::hcl::{Hcl, HclQuery, PremadeHclQuery};

#[rstest]
#[case("variables.tf", HclQuery::Premade(PremadeHclQuery::Variables))]
#[case("resource-types.tf", HclQuery::Premade(PremadeHclQuery::ResourceTypes))]
#[case("resource-names.tf", HclQuery::Premade(PremadeHclQuery::ResourceNames))]
#[case("data-names.tf", HclQuery::Premade(PremadeHclQuery::DataNames))]
#[case("data-sources.tf", HclQuery::Premade(PremadeHclQuery::DataSources))]
#[case("comments.tf", HclQuery::Premade(PremadeHclQuery::Comments))]
#[case("strings.tf", HclQuery::Premade(PremadeHclQuery::Strings))]
fn test_hcl_nuke(#[case] file: &str, #[case] query: HclQuery) {
    let lang = Hcl::new(query);

    let (input, output) = get_input_output("hcl", file);
    let result = nuke_target(&input, &lang);

    assert_eq!(result, output);
}
