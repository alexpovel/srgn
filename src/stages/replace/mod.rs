use log::info;
use unescape::unescape;

use super::Stage;

/// Replaces input with a fixed string.
///
/// ## Example: replacing invalid characters in identifiers
///
/// ```rust,ignore
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
/// ```rust,ignore
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

impl TryFrom<String> for ReplacementStage {
    type Error = String;

    fn try_from(replacement: String) -> Result<Self, Self::Error> {
        let unescaped =
            unescape(&replacement).ok_or("Cannot unescape sequences in replacement".to_string())?;
        Ok(Self {
            replacement: unescaped,
        })
    }
}

impl Stage for ReplacementStage {
    fn process(&self, input: &str) -> String {
        info!("Substituting '{}' with '{}'", input, self.replacement);
        self.replacement.clone()
    }
}
