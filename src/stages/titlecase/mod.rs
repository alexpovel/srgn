use super::Stage;
use titlecase::titlecase;

/// Renders in titlecase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct TitlecaseStage {}

impl Stage for TitlecaseStage {
    fn process(&self, input: &str) -> String {
        titlecase(input)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("a dog", "A Dog")]
    #[case("ein überfall", "Ein Überfall")]
    #[case("miXeD caSe", "miXeD caSe")] // Hmmm... behavior of `titlecase` crate
    //
    #[case("a dog's life 🐕", "A Dog's Life 🐕")]
    //
    #[case("a dime a dozen", "A Dime a Dozen")]
    fn test_titlecasing(#[case] input: &str, #[case] expected: &str) {
        let result = TitlecaseStage::default().process(input);
        assert_eq!(result, expected);
    }
}
