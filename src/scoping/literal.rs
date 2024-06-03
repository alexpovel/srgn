use super::{ROScopes, Scoper};
use crate::ranges::Ranges;
use log::trace;
use std::{error::Error, fmt, ops::Range};
use unescape::unescape;

/// A literal string for querying.
#[derive(Debug)]
pub struct Literal(String);

/// An error that can occur when parsing a literal.
#[derive(Debug)]
pub enum LiteralError {
    /// The literal contains invalid escape sequences.
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

            let ranges: Ranges<usize> = input
                .match_indices(&self.0)
                .map(|(i, _)| Range {
                    start: i,
                    end: i + len,
                })
                .collect();

            trace!("Ranges in scope for {:?}: {:?}", self, ranges);

            ranges
        };

        ROScopes::from_raw_ranges(input, ranges.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoping::{
        scope::{
            RWScope, RWScopes,
            Scope::{In, Out},
        },
        view::ScopedView,
    };
    use rstest::rstest;
    use std::borrow::Cow::Borrowed;

    #[rstest]
    #[case(
        "a",
        "a",
        ScopedView::new(
            RWScopes(vec![
                RWScope(In(Borrowed("a"), None)),
            ])
        )
    )]
    #[case(
        "aa",
        "a",
        ScopedView::new(
            RWScopes(vec![
                RWScope(In(Borrowed("a"), None)),
                RWScope(In(Borrowed("a"), None)),
            ])
        )
    )]
    #[case(
        "aba",
        "a",
        ScopedView::new(
            RWScopes(vec![
                RWScope(In(Borrowed("a"), None)),
                RWScope(Out("b")),
                RWScope(In(Borrowed("a"), None)),
            ])
        )
    )]
    //
    #[case(
        ".",
        ".",
        ScopedView::new(
            RWScopes(vec![
                RWScope(In(Borrowed("."), None)),
            ])
        )
    )]
    #[case(
        r"\.",
        ".",
        ScopedView::new(
            RWScopes(vec![
                RWScope(Out(r"\")),
                RWScope(In(Borrowed("."), None)),
            ])
        )
    )]
    #[case(
        r".",
        r"\\.",
        ScopedView::new(
            RWScopes(vec![
                RWScope(Out(r".")),
            ])
        )
    )]
    //
    #[case(
        "Hello\nWorld\n",
        "\n",
        ScopedView::new(
            RWScopes(vec![
                RWScope(Out("Hello")),
                RWScope(In(Borrowed("\n"), None)),
                RWScope(Out("World")),
                RWScope(In(Borrowed("\n"), None)),
            ])
        )
    )]
    fn test_literal_scoping(
        #[case] input: &str,
        #[case] literal: &str,
        #[case] expected: ScopedView,
    ) {
        let mut builder = crate::scoping::view::ScopedViewBuilder::new(input);
        let literal = Literal::try_from(literal.to_owned()).unwrap();
        builder.explode(&literal);
        let actual = builder.build();

        assert_eq!(actual, expected);
    }
}
