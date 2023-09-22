use regex::Regex;

use super::{langs::python::Scoper, ScopedView};

#[derive(Debug)]
pub struct RegexScoper {
    pattern: Regex,
}

impl RegexScoper {
    #[must_use]
    pub fn new(pattern: Regex) -> Self {
        Self { pattern }
    }
}

impl Scoper for RegexScoper {
    fn scope<'a>(&self, input: &'a str) -> ScopedView<'a> {
        let ranges = self.pattern.find_iter(input).map(|m| m.range());
        ScopedView::from_raw(input, ranges)
    }
}
