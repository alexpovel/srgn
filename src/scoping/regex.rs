use regex::Regex;

use super::{langs::python::Scoper, ranges_to_view};

#[derive(Debug)]
pub struct RegexScoper {
    pattern: Regex,
    next: Option<Box<dyn Scoper>>,
}

impl RegexScoper {
    pub fn new(pattern: Regex, next: Option<Box<dyn Scoper>>) -> Self {
        Self { pattern, next }
    }
}

impl Scoper for RegexScoper {
    fn scope<'a>(&'a self, input: &'a str) -> super::ScopedView {
        ranges_to_view(input, self.pattern.find_iter(input).map(|m| m.range()))
    }

    fn next(self) -> Option<Box<dyn Scoper>> {
        self.next
    }
}
