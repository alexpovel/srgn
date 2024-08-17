use std::ops::Range;

use proptest::prelude::*;
use srgn::ranges::Ranges;

use crate::properties::DEFAULT_NUMBER_OF_TEST_CASES;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(DEFAULT_NUMBER_OF_TEST_CASES))]
    #[test]
    fn test_ranges_from_single_range(
        // https://proptest-rs.github.io/proptest/proptest/tutorial/arbitrary.html
        // https://docs.rs/proptest/latest/proptest/arbitrary/trait.Arbitrary.html#impl-Arbitrary-for-Range%3CA%3E
        range in any::<Range<u16>>(), // `u16` only so we don't allocate half the world
    ) {
        let range = Range::<usize>{
            start: range.start.into(),
            end: range.end.into(),
        };
        let ranges = Ranges::from(&range);

        prop_assert_eq!(ranges.len(), range.len());
    }
}
