mod lower;
mod ranges;
mod squeeze;
#[cfg(feature = "symbols")]
mod symbols;
mod upper;

// https://proptest-rs.github.io/proptest/proptest/tutorial/config.html
const DEFAULT_NUMBER_OF_TEST_CASES: u32 = 512;
