#[cfg(feature = "deletion")]
mod deletion;
#[cfg(feature = "german")]
mod german;
#[cfg(feature = "symbols")]
mod symbols;
/// Tooling (types, traits, ...) around stages.
pub mod tooling;

pub use deletion::DeletionStage;
pub use german::GermanStage;
pub use symbols::SymbolsStage;
pub use tooling::Stage;
