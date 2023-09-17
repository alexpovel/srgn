use betterletters::stages::LowerStage;
use proptest::prelude::*;

use crate::properties::{apply_with_default_scope, DEFAULT_NUMBER_OF_TEST_CASES};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(DEFAULT_NUMBER_OF_TEST_CASES))]
    #[test]
    fn test_lowercasing_lowercase_has_no_effect(
        // https://docs.rs/regex/latest/regex/#matching-one-character
        // https://www.unicode.org/reports/tr44/tr44-24.html#General_Category_Values
        input in r"\p{Lowercase_Letter}*"
    ) {
        assert_eq!(apply_with_default_scope(&LowerStage::default(), &input), input);
    }
}
