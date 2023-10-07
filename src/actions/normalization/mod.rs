use super::Action;
use unicode_categories::UnicodeCategories;
use unicode_normalization::UnicodeNormalization;

/// Performs Unicode normalization.
///
/// Uses NFD (Normalization Form D), canonical decomposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Normalization {}

impl Action for Normalization {
    fn act(&self, input: &str) -> String {
        input.nfd().filter(|c| !c.is_mark()).collect()
    }
}

// #[cfg(test)]
// mod tests {
//     use rstest::rstest;

//     use super::*;

//     #[rstest]
//     #[case("a dog", "A Dog")]
//     #[case("ein überfall", "Ein Überfall")]
//     #[case("miXeD caSe", "miXeD caSe")] // Hmmm... behavior of `titlecase` crate
//     //
//     #[case("a dog's life 🐕", "A Dog's Life 🐕")]
//     //
//     #[case("a dime a dozen", "A Dime a Dozen")]
//     fn test_titlecasing(#[case] input: &str, #[case] expected: &str) {
//         let result = TitlecaseStage::default().process(input);
//         assert_eq!(result, expected);
//     }
// }
