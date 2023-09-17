use log::info;

use crate::scoped::Scoped;

use super::Stage;

/// Replaces input with a fixed string.
///
/// ## Example: replacing invalid characters in identifiers
///
/// ```
/// use betterletters::stages::{Stage, ReplacementStage};
/// use betterletters::scoped::Scope;
/// use regex::Regex;
///
/// let stage = ReplacementStage::new("_".to_string());
/// let scope = Scope::new(Regex::new(r"[^a-zA-Z0-9]+").unwrap());
///
/// assert_eq!(
///    stage.apply("hyphenated-variable-name", &scope),
///   "hyphenated_variable_name"
/// );
/// ```
///
/// ## Example: replace emojis
///
/// ```
/// use betterletters::stages::{Stage, ReplacementStage};
/// use betterletters::scoped::Scope;
/// use regex::Regex;
///
/// let stage = ReplacementStage::new(":(".to_string());
/// // A Unicode character class category. See also
/// // https://github.com/rust-lang/regex/blob/061ee815ef2c44101dba7b0b124600fcb03c1912/UNICODE.md#rl12-properties
/// let scope = Scope::new(Regex::new(r"\p{Emoji}").unwrap());
///
/// assert_eq!(
///    stage.apply("Party! 😁 💃 🎉 🥳 So much fun! ╰(°▽°)╯", &scope),
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
    pub fn new(replacement: String) -> Self {
        Self { replacement }
    }
}

impl Scoped for ReplacementStage {}

impl Stage for ReplacementStage {
    fn substitute(&self, input: &str) -> String {
        info!("Substituting {} with {}", input, self.replacement);
        self.replacement.clone()
    }
}
