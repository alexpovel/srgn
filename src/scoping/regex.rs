use crate::GLOBAL_SCOPE;

use super::{ScopedViewBuildStep, ScopedViewBuilder};

#[derive(Debug)]
pub struct Regex {
    pattern: regex::Regex,
}

impl Regex {
    #[must_use]
    pub fn new(pattern: regex::Regex) -> Self {
        Self { pattern }
    }
}

impl Default for Regex {
    fn default() -> Self {
        Self::new(regex::Regex::new(GLOBAL_SCOPE).unwrap())
    }
}

impl ScopedViewBuildStep for Regex {
    fn scope<'a>(&self, input: &'a str) -> ScopedViewBuilder<'a> {
        ScopedViewBuilder::new(input).explode_from_ranges(|s| {
            let ranges = self.pattern.find_iter(s).map(|m| m.range());

            ranges.collect()
        })
    }
}
