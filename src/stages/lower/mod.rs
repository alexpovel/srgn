use crate::scoped::Scoped;

use super::Stage;

/// Renders in lowercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct LowerStage {}

impl Scoped for LowerStage {}

impl Stage for LowerStage {
    fn substitute(&self, input: &str) -> String {
        input.to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    // German
    #[case("A", "a")]
    #[case("a", "a")]
    #[case("Ä", "ä")]
    #[case("ä", "ä")]
    #[case("Ö", "ö")]
    #[case("ö", "ö")]
    #[case("Ü", "ü")]
    #[case("ü", "ü")]
    #[case("ẞ", "ß")]
    #[case("ß", "ß")]
    #[case("AaÄäÖöÜüẞß!", "aaääööüüßß!")]
    #[case("SS", "ss")]
    //
    // Chinese
    #[case("你好!", "你好!")]
    //
    // Japanese
    #[case("こんにちは!", "こんにちは!")]
    //
    // Korean
    #[case("안녕하세요!", "안녕하세요!")]
    //
    // Russian
    #[case("ПРИВЕТ!", "привет!")]
    //
    // Emojis
    #[case("👋\0", "👋\0")]
    fn substitute(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(LowerStage {}.substitute(input), expected);
    }
}
