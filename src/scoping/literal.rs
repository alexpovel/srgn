use super::{ScopedViewBuildStep, ScopedViewBuilder};
use log::trace;
use std::ops::Range;

#[derive(Debug)]
pub struct Literal(String);

impl Literal {
    #[must_use]
    pub fn new(literal: String) -> Self {
        Self(literal)
    }
}

impl ScopedViewBuildStep for Literal {
    fn scope<'a>(&self, input: &'a str) -> ScopedViewBuilder<'a> {
        ScopedViewBuilder::new(input).explode_from_ranges(|s| {
            let len = self.0.len();

            let ranges = s
                .match_indices(&self.0)
                .map(|(i, _)| Range {
                    start: i,
                    end: i + len,
                })
                .collect();

            trace!("Ranges in scope for {:?}: {:?}", self, ranges);

            ranges
        })
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::scoping::{
        Scope::{In, Out},
        ScopedView,
    };
    use std::borrow::Cow::Borrowed;

    use super::*;

    #[rstest]
    #[case("a", "a", ScopedView::new(vec![In(Borrowed("a"))]))]
    #[case("aa", "a", ScopedView::new(vec![In(Borrowed("a")), In(Borrowed("a"))]))]
    #[case("aba", "a", ScopedView::new(vec![In(Borrowed("a")), Out("b"), In(Borrowed("a"))]))]
    //
    #[case(".", ".", ScopedView::new(vec![In(Borrowed("."))]))]
    #[case(r"\.", ".", ScopedView::new(vec![Out(r"\"), In(Borrowed("."))]))]
    #[case(r".", r"\.", ScopedView::new(vec![Out(r".")]))]
    #[case(r"\.", r"\.", ScopedView::new(vec![In(Borrowed(r"\."))]))]
    #[case(r"\w", r"\w", ScopedView::new(vec![In(Borrowed(r"\w"))]))]
    fn test_literal_scoping(
        #[case] input: &str,
        #[case] literal: &str,
        #[case] expected: ScopedView,
    ) {
        let literal = Literal::new(literal.to_owned());
        let actual = literal.scope(input).build();

        assert_eq!(actual, expected);
    }
}
