use super::{ScopedViewBuildStep, ScopedViewBuilder};
use log::trace;
use std::{error::Error, fmt, ops::Range};
use unescape::unescape;

#[derive(Debug)]
pub struct Literal(String);

#[derive(Debug)]
pub enum LiteralError {
    Inunescapable(String),
}

impl fmt::Display for LiteralError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inunescapable(literal) => write!(f, "Could not unescape literal '{literal}'"),
        }
    }
}

impl Error for LiteralError {}

impl TryFrom<String> for Literal {
    type Error = LiteralError;

    fn try_from(literal: String) -> Result<Self, Self::Error> {
        let unescaped =
            unescape(&literal).ok_or(LiteralError::Inunescapable(literal.to_string()))?;

        Ok(Self(unescaped))
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
        let literal = Literal::try_from(literal.to_owned()).unwrap();
        let actual = literal.scope(input).build();

        assert_eq!(actual, expected);
    }
}
