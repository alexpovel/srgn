#[cfg(feature = "de")]
pub mod german;
#[cfg(feature = "symbols")]
pub mod symbols;

#[derive(Debug)]
pub struct ProcessError;

impl From<ProcessError> for std::io::Error {
    fn from(_: ProcessError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, "Error in text processor.")
    }
}

pub type ProcessResult = Result<(), ProcessError>;

pub trait TextProcessor: Send + Sync {
    fn process(&self, input: &mut String) -> ProcessResult;
}
