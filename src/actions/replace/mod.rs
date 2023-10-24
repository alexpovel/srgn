use log::info;
use unescape::unescape;

use super::Action;

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
pub struct Replacement {
    replacement: String,
}

impl Replacement {
    /// Creates a new replacement.
    #[must_use]
    pub fn new(replacement: String) -> Self {
        Self { replacement }
    }
}

impl TryFrom<String> for Replacement {
    type Error = String;

    fn try_from(replacement: String) -> Result<Self, Self::Error> {
        let unescaped =
            unescape(&replacement).ok_or("Cannot unescape sequences in replacement".to_string())?;
        Ok(Self {
            replacement: unescaped,
        })
    }
}

impl Action for Replacement {
    fn act(&self, input: &str) -> String {
        info!("Substituting '{}' with '{}'", input, self.replacement);
        self.replacement.clone()
    }
}
