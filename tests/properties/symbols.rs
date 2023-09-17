use betterletters::stages::{SymbolsInversionStage, SymbolsStage};
use proptest::prelude::*;

use crate::properties::{apply_with_default_scope, DEFAULT_NUMBER_OF_TEST_CASES};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(DEFAULT_NUMBER_OF_TEST_CASES * 2))]
    #[test]
    fn test_inverting_symbols_is_idempotent(
        // https://docs.rs/regex/latest/regex/#matching-one-character
        // https://www.unicode.org/reports/tr44/tr44-24.html#General_Category_Values
        input in r"\p{Any}*(-|<|>|=|!){2,3}\p{Any}*"
    ) {
        let applied = apply_with_default_scope(&SymbolsStage::default(), &input);
        let inverted = apply_with_default_scope(&SymbolsInversionStage::default(), &applied);

        assert_eq!(input, inverted);
    }
}
