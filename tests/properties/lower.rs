use proptest::prelude::*;
use srgn::{
    scoping::ScopedViewBuilder,
    stages::{LowerStage, Stage},
};

use crate::properties::DEFAULT_NUMBER_OF_TEST_CASES;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(DEFAULT_NUMBER_OF_TEST_CASES))]
    #[test]
    fn test_lowercasing_lowercase_has_no_effect(
        // https://docs.rs/regex/latest/regex/#matching-one-character
        // https://www.unicode.org/reports/tr44/tr44-24.html#General_Category_Values
        input in r"\p{Lowercase_Letter}*"
    ) {
        let stage = LowerStage::default();
        let mut view = ScopedViewBuilder::new(&input).build();
        stage.map(&mut view);
        let res = view.to_string();

        assert_eq!(res, input);
    }
}
