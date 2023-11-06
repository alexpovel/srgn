use super::Action;
use log::info;
use std::{error::Error, fmt};
use unescape::unescape;

/// Replaces input with a fixed string.
///
/// ## Example: replacing invalid characters in identifiers
///
/// ```rust
/// use srgn::RegexPattern;
/// use srgn::scoping::{view::ScopedViewBuilder, regex::Regex};
///
/// let scoper = Regex::new(RegexPattern::new(r"[^a-zA-Z0-9]+").unwrap());
/// let mut builder = ScopedViewBuilder::new("hyphenated-variable-name");
/// builder.explode(&scoper);
/// let mut view = builder.build();
/// view.replace("_".to_string());
///
/// assert_eq!(
///    view.to_string(),
///   "hyphenated_variable_name"
/// );
/// ```
///
/// ## Example: replace emojis
///
/// ```rust
/// use srgn::RegexPattern;
/// use srgn::scoping::{view::ScopedViewBuilder, regex::Regex};
///
/// // A Unicode character class category. See also
/// // https://github.com/rust-lang/regex/blob/061ee815ef2c44101dba7b0b124600fcb03c1912/UNICODE.md#rl12-properties
/// let scoper = Regex::new(RegexPattern::new(r"\p{Emoji}").unwrap());
/// let mut builder = ScopedViewBuilder::new("Party! 😁 💃 🎉 🥳 So much fun! ╰(°▽°)╯");
/// builder.explode(&scoper);
/// let mut view = builder.build();
/// view.replace(":(".to_string());
///
/// assert_eq!(
///    view.to_string(),
///    // Party is over, sorry ¯\_(ツ)_/¯
///   "Party! :( :( :( :( So much fun! ╰(°▽°)╯"
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Replacement(String);

impl Replacement {
    /// Creates a new replacement.
    #[must_use]
    pub fn new(replacement: String) -> Self {
        Self(replacement)
    }
}

impl TryFrom<String> for Replacement {
    type Error = ReplacementCreationError;

    fn try_from(replacement: String) -> Result<Self, Self::Error> {
        match unescape(&replacement) {
            Some(res) => Ok(Self(res)),
            None => Err(ReplacementCreationError::InvalidEscapeSequences(
                replacement,
            )),
        }
    }
}

/// An error that can occur when creating a replacement.
#[derive(Debug, PartialEq, Eq)]
pub enum ReplacementCreationError {
    /// The replacement contains invalid escape sequences.
    InvalidEscapeSequences(String),
}

impl fmt::Display for ReplacementCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEscapeSequences(replacement) => {
                write!(f, "Contains invalid escape sequences: '{replacement}'")
            }
        }
    }
}

impl Error for ReplacementCreationError {}

impl Action for Replacement {
    fn act(&self, input: &str) -> String {
        info!("Substituting '{}' with '{}'", input, self.0);
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(r"Some Replacement", Ok(Replacement("Some Replacement".to_string())))]
    #[case(r"Some \t Escape", Ok(Replacement("Some \t Escape".to_string())))]
    #[case(r"Some Invalid \z Escape", Err(ReplacementCreationError::InvalidEscapeSequences(r"Some Invalid \z Escape".to_string())))]
    fn test_replacement_creation(
        #[case] input: &str,
        #[case] result: Result<Replacement, ReplacementCreationError>,
    ) {
        let replacement = Replacement::try_from(input.to_string());
        assert_eq!(replacement, result);
    }
}
