#[cfg(feature = "de")]
pub mod german;
#[cfg(feature = "symbols")]
pub mod symbols;

pub trait TextProcessor {
    fn process(&self, input: &mut String) -> bool;
}
