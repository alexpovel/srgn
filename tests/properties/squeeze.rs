use betterletters::{scoped::Scope, stages::SqueezeStage};
use proptest::prelude::*;
use regex::Regex;

use crate::properties::{apply, DEFAULT_NUMBER_OF_TEST_CASES};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(DEFAULT_NUMBER_OF_TEST_CASES))]
    #[test]
    fn test_squeezing_anything_at_all_makes_the_input_shorter(
        // https://docs.rs/regex/latest/regex/#matching-one-character
        // https://www.unicode.org/reports/tr44/tr44-24.html#General_Category_Values
        input in r"\p{Any}*AA\p{Any}*"
    ) {
        let scope = Scope::from(Regex::new("A").unwrap());
        assert!(apply(&SqueezeStage::default(), &input, scope).len() < input.len());
    }
}
