use betterletters::{scoping::ScopedView, stages::SqueezeStage, Stage};
use proptest::prelude::*;

use crate::properties::DEFAULT_NUMBER_OF_TEST_CASES;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(DEFAULT_NUMBER_OF_TEST_CASES))]
    #[test]
    fn test_squeezing_anything_at_all_makes_the_input_shorter(
        // https://docs.rs/regex/latest/regex/#matching-one-character
        // https://www.unicode.org/reports/tr44/tr44-24.html#General_Category_Values
        input in r"\p{Any}*AA\p{Any}*"
    ) {
        let stage = SqueezeStage::default();
        let mut view = ScopedView::new(&input);

        stage.substitute(&mut view);
        let res = view.to_string();

        assert!(res.len() < input.len());
    }
}
