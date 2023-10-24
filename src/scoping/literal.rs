use super::{ROScopes, Scoper};
use log::trace;
use std::{error::Error, fmt, ops::Range};
use unescape::unescape;

#[derive(Debug)]
pub struct Literal(String);

#[derive(Debug)]
pub enum LiteralError {
    InvalidEscapeSequences(String),
}

impl fmt::Display for LiteralError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEscapeSequences(literal) => {
                write!(f, "Contains invalid escape sequences: '{literal}'")
            }
        }
    }
}

impl Error for LiteralError {}

impl TryFrom<String> for Literal {
    type Error = LiteralError;

    fn try_from(literal: String) -> Result<Self, Self::Error> {
        let unescaped =
            unescape(&literal).ok_or(LiteralError::InvalidEscapeSequences(literal.to_string()))?;

        Ok(Self(unescaped))
    }
}

impl Scoper for Literal {
    fn scope<'viewee>(&self, input: &'viewee str) -> ROScopes<'viewee> {
        let ranges = {
            let len = self.0.len();

            let ranges = input
                .match_indices(&self.0)
                .map(|(i, _)| Range {
                    start: i,
                    end: i + len,
                })
                .collect();

            trace!("Ranges in scope for {:?}: {:?}", self, ranges);

            ranges
        };

        ROScopes::from_raw_ranges(input, ranges)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::scoping::{
        scope::{
            RWScope, RWScopes,
            Scope::{In, Out},
        },
        view::ScopedView,
    };
    use std::borrow::Cow::Borrowed;

    use super::*;

    #[rstest]
    #[case("a", "a", ScopedView::new(RWScopes(vec![RWScope(In(Borrowed("a")))])))]
    #[case("aa", "a", ScopedView::new(RWScopes(vec![RWScope(In(Borrowed("a"))), RWScope(In(Borrowed("a")))])))]
    #[case("aba", "a", ScopedView::new(RWScopes(vec![RWScope(In(Borrowed("a"))), RWScope(Out("b")), RWScope(In(Borrowed("a")))])))]
    //
    #[case(".", ".", ScopedView::new(RWScopes(vec![RWScope(In(Borrowed(".")))])))]
    #[case(r"\.", ".", ScopedView::new(RWScopes(vec![RWScope(Out(r"\")), RWScope(In(Borrowed(".")))])))]
    #[case(r".", r"\\.", ScopedView::new(RWScopes(vec![RWScope(Out(r"."))])))]
    //
    #[case("Hello\nWorld\n", "\n", ScopedView::new(RWScopes(vec![RWScope(Out("Hello")), RWScope(In(Borrowed("\n"))), RWScope(Out("World")), RWScope(In(Borrowed("\n")))])))]
    fn test_literal_scoping(
        #[case] input: &str,
        #[case] literal: &str,
        #[case] expected: ScopedView,
    ) {
        let builder = crate::scoping::view::ScopedViewBuilder::new(input);
        let literal = Literal::try_from(literal.to_owned()).unwrap();
        let actual = builder.explode(&literal).build();

        assert_eq!(actual, expected);
    }
}
