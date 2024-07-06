use crate::{ranges::Ranges, scoping::Scoper};
use colored::Colorize;
use itertools::Itertools;
use log::{debug, warn};
use ranges::{GlobalRange, LocalRange};
use std::{marker::PhantomData, ops::Range};

/// A line to be printed to a destination, with relevant metadata.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Line<'a, D: Destination> {
    /// The line number.
    number: usize,
    /// The *entire* line contents.
    content: &'a str,
    /// Potential highlights to apply to this line, as indices into the `content`.
    highlights: Option<Ranges<usize>>,
    /// <https://cliffle.com/blog/rust-typestate/#variation-state-type-parameter>
    _marker: PhantomData<D>,
}

impl<'a, D: Destination> Line<'a, D> {
    /// Return whether this line contains any highlights.
    #[must_use]
    pub fn has_highlights(&self) -> bool {
        self.highlights.is_some()
    }
}

/// A type-state trait for modelling an output destination.
pub trait Destination: std::fmt::Debug + Default {}

/// A teletype.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Tty;

/// Not a teletype; opposite of [`Tty`].
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct NoTty;

impl Destination for Tty {}
impl Destination for NoTty {}

impl<'a> std::fmt::Display for Line<'a, Tty> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(highlights) = &self.highlights {
            write!(f, "{}:", self.number.to_string().green())?;

            let mut last_end = 0;
            for highlight in highlights {
                write!(f, "{}", &self.content[last_end..highlight.start])?;
                write!(f, "{}", &self.content[highlight.clone()].red())?;
                last_end = highlight.end;
            }

            write!(f, "{}", &self.content[last_end..])?;
        };

        Ok(())
    }
}

impl<'a> std::fmt::Display for Line<'a, NoTty> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.highlights.is_some() {
            write!(f, "{}:", self.number.to_string().green())?;
            write!(f, "{}", self.content)?;
        }

        Ok(())
    }
}

/// Like normal `grep`, takes some input, and scopes it down using the regular
/// [`Scoper`] mechanisms, allowing this function to do more than regex.
#[must_use]
#[allow(clippy::missing_panics_doc)] // Internal asserts, not for public consumption
pub fn grep<'viewee, D: Destination>(
    input: &'viewee str,
    scopers: &[Box<dyn Scoper>],
) -> Vec<Line<'viewee, D>> {
    let mut ranges = vec![GlobalRange::from(0..input.len())];
    debug!("Initial ranges: {ranges:?}");

    for scoper in scopers {
        ranges = ranges
            .into_iter()
            .flat_map(|global_range| {
                scoper
                    .scope_raw(&input[Range::from(global_range)])
                    .into_iter()
                    // ⚠️ This shift from types back and forth is mainly cosmetic, but
                    // communicates intent. It's not fully safe, as `scope_raw` works
                    // with normal `std::ops::Range`, which we take as identical to
                    // 'local ranges', hence convert back and forth once.
                    //
                    // TODO: implement `LocalRange` across the entire crate?
                    .map(|(r, _)| LocalRange::new(r, global_range.start))
                    .map(GlobalRange::from)
                    .collect_vec()
            })
            .collect();
        debug!("Ranges after scoping: {ranges:?}");
    }

    if scopers.is_empty() {
        // Can't have been any
        warn!("No scopers, will not highlight any lines");
        ranges.clear();
    } else {
        ranges.retain(|range| !range.is_empty());
        ranges.sort_by_key(|r| r.start);
    }
    debug!("Scoped ranges to highlight: {:?}", ranges);

    let mut ranges = ranges.into_iter();
    let mut range = ranges.next();
    let mut lines = Vec::new();

    let mut line = GlobalRange::from(0..0);
    for (i, contents) in input.split_inclusive(['\n', '\r']).enumerate() {
        let i = i + 1;

        line.end = line.start + contents.len();

        let mut highlights = Vec::new();

        while let Some(GlobalRange { start, end }) = range {
            assert!(start >= line.start, "Previous iteration sliced incorrectly");

            if start >= line.end {
                // Ranges are sorted and we're beyond this line's range; go next line.
                break;
            }

            assert!(line.contains(start));

            if end <= line.end {
                // Range is fully contained in current line; push and go next range.
                highlights.push(GlobalRange::from(start..end).shift(line.start));
                range = ranges.next();
                continue;
            }

            // There's a split; split the current `range` up over lines
            assert!(!line.contains(end));
            highlights.push(GlobalRange::from(start..line.end).shift(line.start));
            range = Some(GlobalRange::from(line.end..end));
            break;
        }

        assert!(
            match lines.last() {
                Some(Line { number, .. }) => number + 1 == i,
                None => true,
            },
            "Non-consecutive lines pushed: '{}' after '{:?}'",
            contents.escape_debug(),
            lines.last(),
        );

        lines.push(Line {
            number: i,
            content: contents,
            highlights: if highlights.is_empty() {
                None
            } else {
                assert!(
                    highlights.iter().all(|h| contents.get(h.clone()).is_some()),
                    "Highlights range '{highlights:?}' out of range for line '{}'",
                    contents.escape_debug()
                );

                Some(Ranges::from_iter(highlights))
            },
            ..Default::default()
        });

        line.start = line.end;
    }

    lines
}

/// Module for (very leaky) abstractions over [`std::ops::Range`] to model global and
/// local ranges, for increased type safety in handling (instead of raw `usize`s).
pub mod ranges {
    use std::ops::{Add, Range, Sub};

    /// A range modelled after [`Range`], where [`GlobalRange::start`] and
    /// [`GlobalRange::end`] denote *global* positions, across the entire input.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
    pub struct GlobalRange<T = usize> {
        pub start: T,
        pub end: T,
    }

    impl GlobalRange {
        pub fn is_empty(&self) -> bool {
            Range::from(self.clone()).is_empty()
        }

        pub fn contains(&self, item: usize) -> bool {
            Range::from(self.clone()).contains(&item)
        }

        /// Create a new [`Range`] by shifting by the given `offset`.
        ///
        /// A [`GlobalRange`] of `5..8` with an `offset` of `2` will become `3..6`.
        pub fn shift<T>(self, by: T) -> Range<T>
        where
            T: Copy,
            usize: Sub<T, Output = T>,
        {
            Range {
                start: self.start - by,
                end: self.end - by,
            }
        }
    }

    impl<T> From<GlobalRange<T>> for Range<T> {
        fn from(range: GlobalRange<T>) -> Self {
            Self {
                start: range.start,
                end: range.end,
            }
        }
    }

    impl<T> From<Range<T>> for GlobalRange<T> {
        fn from(range: Range<T>) -> Self {
            Self {
                start: range.start,
                end: range.end,
            }
        }
    }

    /// A range local to some offset from the global origin.
    ///
    /// A [`LocalRange`] of `2..3` with an [`LocalRange::offset`] of `5` corresponds to
    /// a [`GlobalRange`] of `7..8` (see [`GlobalRange::from`] for [`LocalRange`]).
    #[derive(Debug, Clone)]
    pub struct LocalRange<T = usize> {
        start: T,
        end: T,
        offset: T,
    }

    impl<T> From<LocalRange<T>> for GlobalRange<T>
    where
        T: Add<T, Output = T> + Copy,
    {
        fn from(range: LocalRange<T>) -> Self {
            Self {
                start: range.offset + range.start,
                end: range.offset + range.end,
            }
        }
    }

    impl LocalRange {
        /// Create a new instance, with the specified `offset`.
        pub fn new(range: Range<usize>, offset: usize) -> Self {
            Self {
                start: range.start,
                end: range.end,
                offset,
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::single_range_in_vec_init)]
mod tests {
    use super::*;
    use crate::scoping::langs::python::{PreparedPythonQuery, Python};
    use crate::scoping::langs::CodeQuery;
    use crate::scoping::{regex::Regex, Scoper};
    use crate::RegexPattern;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    fn make_regex(pattern: &str) -> Box<dyn Scoper> {
        Box::new(Regex::new(RegexPattern::new(pattern).unwrap()))
    }

    fn make_python_comments() -> Box<dyn Scoper> {
        Box::new(Python::new(CodeQuery::Prepared(
            PreparedPythonQuery::Comments,
        )))
    }

    #[allow(clippy::unnecessary_wraps)]
    fn make_highlights<T, const N: usize>(highlights: [Range<T>; N]) -> Option<Ranges<T>>
    where
        T: Ord + Copy + std::fmt::Debug,
    {
        Some(Ranges::from_iter(highlights))
    }

    #[rstest]
    #[case(
        // Empty everything
        "",
        vec![
        ],
        vec![
        ],
    )]
    #[case(
        // No scoper, so no highlights
        "hello",
        vec![
        ],
        vec![
            Line {
                number: 1,
                content: "hello",
                highlights: None,
                ..Default::default()
            },
        ],
    )]
    #[case(
        // Regex matches nothing
        "hello",
        vec![
            make_regex("world"),
        ],
        vec![
            Line {
                number: 1,
                content: "hello",
                highlights: None,
                ..Default::default()
            },
        ],
    )]
    #[case(
        // Regex scopes all
        "hello",
        vec![
            make_regex("hello"),
        ],
        vec![
            Line {
                number: 1,
                content: "hello",
                highlights: make_highlights([0..5]),
                ..Default::default()
            },
        ],
    )]
    #[case(
        // Regex scopes part
        "hello world",
        vec![
            make_regex("llo"),
        ],
        vec![
            Line {
                number: 1,
                content: "hello world",
                highlights: make_highlights([2..5]),
                ..Default::default()
            },
        ],
    )]
    #[case(
        // Second regex narrows scope for highlighting
        "hello world",
        vec![
            make_regex("hello"),
            make_regex("lo"),
        ],
        vec![
            Line {
                number: 1,
                content: "hello world",
                highlights: make_highlights([3..5]),
                ..Default::default()
            },
        ],
    )]
    #[case(
        // Second regex ends up matching nothing
        "hello world",
        vec![
            make_regex("hello"),
            make_regex(""),
        ],
        vec![
            Line {
                number: 1,
                content: "hello world",
                highlights: None,
                ..Default::default()
            },
        ],
    )]
    #[case(
        // Second regex ends up matching anything
        "hello world",
        vec![
            make_regex("hello"),
            make_regex(".+"),
        ],
        vec![
            Line {
                number: 1,
                content: "hello world",
                highlights: make_highlights([0..5]),
                ..Default::default()
            },
        ],
    )]
    #[case(
        // More than one line
        "hello\nworld",
        vec![
        ],
        vec![
            Line {
                number: 1,
                content: "hello\n",
                highlights: None,
                ..Default::default()
            },
            Line {
                number: 2,
                content: "world",
                highlights: None,
                ..Default::default()
            },
        ],
    )]
    #[case(
        // More than one line, with one matching
        "hello\nworld",
        vec![
            make_regex("hello")
        ],
        vec![
            Line {
                number: 1,
                content: "hello\n",
                highlights: make_highlights([0..5]),
                ..Default::default()
            },
            Line {
                number: 2,
                content: "world",
                highlights: None,
                ..Default::default()
            },
        ],
    )]
    #[case(
        // More than one line, with both matching
        "hello\nworld",
        vec![
            make_regex("hello|world")
        ],
        vec![
            Line {
                number: 1,
                content: "hello\n",
                highlights: make_highlights([0..5]),
                ..Default::default()
            },
            Line {
                number: 2,
                content: "world",
                highlights: make_highlights([0..5]),
                ..Default::default()
            },
        ],
    )]
    #[case(
        // More than one line, with empty lines, with some matching
        "hello\n\n\nworld",
        vec![
            make_regex("hello|world")
        ],
        vec![
            Line {
                number: 1,
                content: "hello\n",
                highlights: make_highlights([0..5]),
                ..Default::default()
            },
            Line {
                number: 2,
                content: "\n",
                highlights: None,
                ..Default::default()
            },
            Line {
                number: 3,
                content: "\n",
                highlights: None,
                ..Default::default()
            },
            Line {
                number: 4,
                content: "world",
                highlights: make_highlights([0..5]),
                ..Default::default()
            },
        ],
    )]
    #[case(
        // Empty lines only
        "\n\n\n",
        vec![
            make_regex("hello")
        ],
        vec![
            Line {
                number: 1,
                content: "\n",
                highlights: None,
                ..Default::default()
            },
            Line {
                number: 2,
                content: "\n",
                highlights: None,
                ..Default::default()
            },
            Line {
                number: 3,
                content: "\n",
                highlights: None,
                ..Default::default()
            },
        ],
    )]
    #[case(
        // Regex across lines (multi-line mode)
        "hello\nworld",
        vec![
            make_regex("(?m)hello\nworld")
        ],
        vec![
            Line {
                number: 1,
                content: "hello\n",
                highlights: make_highlights([0..6]), // ⚠️ newline included in range
                ..Default::default()
            },
            Line {
                number: 2,
                content: "world",
                highlights: make_highlights([0..5]),
                ..Default::default()
            },
        ],
    )]
    #[case(
        // Regex across lines (multi-line mode), partial lines
        "hello\nworld",
        vec![
            make_regex("(?m)lo\nwo")
        ],
        vec![
            Line {
                number: 1,
                content: "hello\n",
                highlights: make_highlights([3..6]), // ⚠️ newline included in range
                ..Default::default()
            },
            Line {
                number: 2,
                content: "world",
                highlights: make_highlights([0..2]),
                ..Default::default()
            },
        ],
    )]
    #[allow(clippy::needless_raw_string_hashes)] // Wanna treat things identically for easy multi-cursor editing
    #[case(
        // With a language
        r#""""GNU module."""

def GNU_says_moo():
    """The GNU -> say moo -> ✅"""

    GNU = """
      GNU
    """  # the GNU...

    GNU_says_moo(GNU + " says moo")  # ...say moo
"#,
        vec![
            make_python_comments(),
            make_regex("GNU"),
        ],
        vec![
            Line {
                number: 1,
                content: "\"\"\"GNU module.\"\"\"\n",
                highlights: None,
                ..Default::default()
            },
            Line {
                number: 2,
                content: "\n",
                highlights: None,
                ..Default::default()
            },
            Line {
                number: 3,
                content: "def GNU_says_moo():\n",
                highlights: None,
                ..Default::default()
            },
            Line {
                number: 4,
                content: "    \"\"\"The GNU -> say moo -> ✅\"\"\"\n",
                highlights: None,
                ..Default::default()
            },
            Line {
                number: 5,
                content: "\n",
                highlights: None,
                ..Default::default()
            },
            Line {
                number: 6,
                content: "    GNU = \"\"\"\n",
                highlights: None,
                ..Default::default()
            },
            Line {
                number: 7,
                content: "      GNU\n",
                highlights: None,
                ..Default::default()
            },
            Line {
                number: 8,
                content: "    \"\"\"  # the GNU...\n",
                highlights: make_highlights([15..18]),
                ..Default::default()
            },
            Line {
                number: 9,
                content: "\n",
                highlights: None,
                ..Default::default()
            },
            Line {
                number: 10,
                content: "    GNU_says_moo(GNU + \" says moo\")  # ...say moo\n",
                highlights: None,
                ..Default::default()
            },
        ],
    )]
    fn test_grep(
        #[case] input: &str,
        #[case] scopers: Vec<Box<dyn Scoper>>,
        #[case] expected: Vec<Line<'_, NoTty>>,
    ) {
        assert_eq!(grep(input, &scopers), expected);
    }
}
