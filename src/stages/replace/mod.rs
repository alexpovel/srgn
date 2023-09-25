use log::info;

use super::Stage;

/// Replaces input with a fixed string.
///
/// ## Example: replacing invalid characters in identifiers
///
/// ```
/// use betterletters::RegexPattern;
/// use betterletters::stages::{Stage, ReplacementStage};
/// use betterletters::scoping::{ScopedViewBuilder, regex::Regex};
///
/// let stage = ReplacementStage::new("_".to_string());
/// let scoper = Regex::new(RegexPattern::new(r"[^a-zA-Z0-9]+").unwrap());
/// let mut view = ScopedViewBuilder::new("hyphenated-variable-name").explode_from_scoper(
///     &scoper
/// ).build();
///
/// assert_eq!(
///    stage.map(&mut view).to_string(),
///   "hyphenated_variable_name"
/// );
/// ```
///
/// ## Example: replace emojis
///
/// ```
/// use betterletters::RegexPattern;
/// use betterletters::stages::{Stage, ReplacementStage};
/// use betterletters::scoping::{ScopedViewBuilder, regex::Regex};
///
/// let stage = ReplacementStage::new(":(".to_string());
/// // A Unicode character class category. See also
/// // https://github.com/rust-lang/regex/blob/061ee815ef2c44101dba7b0b124600fcb03c1912/UNICODE.md#rl12-properties
/// let scoper = Regex::new(RegexPattern::new(r"\p{Emoji}").unwrap());
/// let mut view = ScopedViewBuilder::new("Party! 😁 💃 🎉 🥳 So much fun! ╰(°▽°)╯").explode_from_scoper(
///     &scoper
/// ).build();
///
/// assert_eq!(
///    stage.map(&mut view).to_string(),
///    // Party is over, sorry ¯\_(ツ)_/¯
///   "Party! :( :( :( :( So much fun! ╰(°▽°)╯"
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplacementStage {
    replacement: String,
}

impl ReplacementStage {
    /// Creates a new `ReplacementStage`.
    #[must_use]
    pub fn new(mut replacement: String) -> Self {
        replacement = substitute_escape_sequences(&replacement);

        Self { replacement }
    }
}

impl Stage for ReplacementStage {
    fn process(&self, input: &str) -> String {
        info!("Substituting '{}' with '{}'", input, self.replacement);
        self.replacement.clone()
    }
}

/// Replaces literal escape sequences with their corresponding, actual characters.
fn substitute_escape_sequences(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                let replacement = match next {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    _ => {
                        result.push('\\');
                        next
                    }
                };
                result.push(replacement);
            } else {
                result.push('\\');
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("a", "a")]
    #[case(r"\a", r"\a")]
    //
    #[case(r"\n", "\n")]
    #[case(r"\r", "\r")]
    #[case(r"\t", "\t")]
    #[case(r"\", r"\")]
    #[case(r"\\", r"\\")]
    //
    #[case(r"a\n", "a\n")]
    #[case(r"a\r", "a\r")]
    #[case(r"a\t", "a\t")]
    #[case(r"a\\", r"a\\")]
    //
    #[case(r"\na", "\na")]
    #[case(r"\ra", "\ra")]
    #[case(r"\ta", "\ta")]
    #[case(r"\\a", r"\\a")]
    //
    #[case(r"a\n\r\t\\", "a\n\r\t\\\\")]
    #[case(r"\n\r\t\\a", "\n\r\t\\\\a")]
    //
    fn test_replace_literal_escape_sequences(#[case] input: &str, #[case] expected: &str) {
        let result = substitute_escape_sequences(input);
        assert_eq!(result, expected);
    }
}
