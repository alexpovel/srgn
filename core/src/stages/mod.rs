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

pub type StageResult = Result<(), StageError>;

pub trait Stage: Send + Sync {
    fn process(&self, input: &mut String) -> StageResult;
}
