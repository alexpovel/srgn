#[cfg(feature = "lower")]
mod lower;
#[cfg(feature = "squeeze")]
mod squeeze;
#[cfg(feature = "symbols")]
mod symbols;
#[cfg(feature = "upper")]
mod upper;

// https://proptest-rs.github.io/proptest/proptest/tutorial/config.html
const DEFAULT_NUMBER_OF_TEST_CASES: u32 = 512;
