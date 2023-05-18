pub mod german;

pub trait TextProcessor {
    fn process(&self, input: &mut String) -> bool;
}
