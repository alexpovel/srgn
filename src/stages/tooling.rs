use std::error::Error;

/// An error that occurred during processing in a stage.
#[derive(Debug, Copy, Clone)]
pub struct StageError;

impl From<StageError> for std::io::Error {
    fn from(e: StageError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    }
}

impl Error for StageError {}

impl std::fmt::Display for StageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error in processing stage")
    }
}

/// A string that has been substituted by a stage.
///
/// This is a
/// [newtype](https://doc.rust-lang.org/rust-by-example/generics/new_types.html), used
/// for increased clarity.
#[derive(Debug)]
pub struct SubstitutedString(String);

/// Convert a [`SubstitutedString`] into a [`String`].
///
/// Convenience method.
impl From<SubstitutedString> for String {
    fn from(s: SubstitutedString) -> Self {
        s.0
    }
}

/// Convert a [`String`] into a [`SubstitutedString`].
///
/// Convenience method.
impl From<String> for SubstitutedString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// The [`Result`] of a stage: we either [substituted properly][SubstitutedString], or [failed][StageError].
pub type StageResult = Result<SubstitutedString, StageError>;

/// A stage in the processing pipeline, as initiated by [`crate::apply`].
///
/// Stages are the core of the text processing pipeline and can be applied in any order,
/// [any number of times each](https://en.wikipedia.org/wiki/Idempotence) (more than
/// once being wasted work, though).
pub trait Stage: Send + Sync {
    /// Substitute text in a given `input` string.
    ///
    /// # Errors
    ///
    /// This method can error out if the stage fails to process the input.
    fn substitute(&self, input: &str) -> StageResult;
}
