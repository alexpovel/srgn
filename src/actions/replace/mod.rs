use super::{Action, ActionError};
use crate::scoping::scope::ScopeContext;
use log::{debug, info};
use std::{error::Error, fmt};
use unescape::unescape;
use variables::{inject_variables, VariableExpressionError};

mod variables;

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

impl TryFrom<String> for Replacement {
    type Error = ReplacementError;

    /// Creates a new replacement from an owned string.
    ///
    /// Escape sequences are accepted and processed, with invalid escape sequences
    /// returning an [`Err`].
    ///
    /// ## Example: Basic usage
    ///
    /// ```
    /// use srgn::actions::Replacement;
    ///
    /// // Successful creation of a regular string
    /// let replacement = Replacement::try_from("Some Replacement".to_owned());
    /// assert!(replacement.is_ok());
    ///
    /// // Successful creation, with escape characters
    /// let replacement = Replacement::try_from(r"Some \t Escape".to_owned());
    /// assert!(replacement.is_ok());
    /// ```
    ///
    /// ## Example: Invalid escape sequence
    ///
    /// Creation fails due to invalid escape sequences.
    ///
    /// ```
    /// use srgn::actions::{Replacement, ReplacementError};
    ///
    /// let replacement = Replacement::try_from(r"Invalid \z Escape".to_owned());
    /// assert_eq!(
    ///    replacement,
    ///    Err(ReplacementError::InvalidEscapeSequences(
    ///      "Invalid \\z Escape".to_owned()
    ///    ))
    /// );
    /// ```
    fn try_from(replacement: String) -> Result<Self, Self::Error> {
        let unescaped =
            unescape(&replacement).ok_or(ReplacementError::InvalidEscapeSequences(replacement))?;

        Ok(Self(unescaped))
    }
}

/// An error that can occur when creating a replacement.
#[derive(Debug, PartialEq, Eq)]
pub enum ReplacementError {
    /// The replacement contains invalid escape sequences.
    InvalidEscapeSequences(String),
    /// The replacement contains an error in its variable expressions.
    VariableError(VariableExpressionError),
}

impl fmt::Display for ReplacementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEscapeSequences(replacement) => {
                write!(f, "Contains invalid escape sequences: '{replacement}'")
            }
            Self::VariableError(err) => {
                write!(f, "Error in variable expressions: {err}")
            }
        }
    }
}

impl Error for ReplacementError {}

impl From<VariableExpressionError> for ReplacementError {
    fn from(value: VariableExpressionError) -> Self {
        Self::VariableError(value)
    }
}

impl Action for Replacement {
    fn act(&self, input: &str) -> String {
        info!("Substituting '{}' with '{}'", input, self.0);
        info!("This substitution is verbatim and does not take into account variables");
        self.0.clone()
    }

    fn act_with_context(
        &self,
        _input: &str,
        context: &ScopeContext,
    ) -> Result<String, ActionError> {
        match context {
            ScopeContext::CaptureGroups(cgs) => {
                debug!("Available capture group variables: {cgs:?}");

                Ok(inject_variables(self.0.as_str(), cgs)?)
            }
        }
    }
}

impl From<VariableExpressionError> for ActionError {
    fn from(value: VariableExpressionError) -> Self {
        Self::ReplacementError(value.into())
    }
}

impl From<ReplacementError> for ActionError {
    fn from(value: ReplacementError) -> Self {
        Self::ReplacementError(value)
    }
}
