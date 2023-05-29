#[cfg(feature = "de")]
pub mod german;
#[cfg(feature = "symbols")]
pub mod symbols;

#[derive(Debug)]
pub struct StageError;

impl From<StageError> for std::io::Error {
    fn from(_: StageError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, "Error in text processor.")
    }
}

#[derive(Debug)]
pub struct SubstitutedString(pub String);

impl From<SubstitutedString> for String {
    fn from(s: SubstitutedString) -> Self {
        s.0
    }
}

impl From<String> for SubstitutedString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

pub type StageResult = Result<SubstitutedString, StageError>;

pub trait Stage: Send + Sync {
    fn substitute(&self, input: &str) -> StageResult;
}
