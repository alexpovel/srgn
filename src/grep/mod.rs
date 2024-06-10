use log::{debug, warn};

use crate::{ranges::Ranges, scoping::Scoper};
use std::{
    collections::HashMap,
    ops::{Range, Sub},
    path::Path,
};

// #[derive(Debug, Clone, PartialEq, Eq)]
// enum Source<'f> {
//     Stdin,
//     File(&'f Path),
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// struct Hits<'f, 'viewee> {
//     source: Source<'f>,
//     lines: Vec<Line<'viewee>>,
// }

#[derive(Debug, Clone, PartialEq, Eq)]
struct Line<'a> {
    number: usize,
    content: &'a str,
    highlights: Option<Ranges<usize>>,
}

// enum Rangee<T> {
//     Local(Range<T>),
//     Global(Range<T>),
// }

// struct ContextualRange<C: Context, T = usize> {
//     start: T,
//     end: T,
//     _marker: PhantomData<C>,
// }

// impl<C: Context, T> ContextualRange<C, T> {
//     fn new(range: Range<T>) -> Self {
//         Self {
//             start: range.start,
//             end: range.end,
//             _marker: PhantomData,
//         }
//     }
// }

// impl<T> From<ContextualRange<Global, T>> for Range<T> {
//     fn from(range: ContextualRange<Global, T>) -> Self {
//         Range {
//             start: range.start,
//             end: range.end,
//         }
//     }
// }

// struct Global {}
// struct Local {}

// trait Context {}

// impl Context for Global {}
// impl Context for Local {}

fn relative_to<T, B>(base: B, r: Range<T>) -> Range<T>
where
    T: Sub<B, Output = T>,
    B: Copy,
{
    Range {
        start: r.start - base,
        end: r.end - base,
    }
}

fn grep<'viewee>(input: &'viewee str, scopers: &Vec<Box<dyn Scoper>>) -> Vec<Line<'viewee>> {
    #[allow(clippy::single_range_in_vec_init)]
    let mut ranges = vec![0..input.len()];

    for scoper in scopers {
        ranges = ranges
            .into_iter()
            .map(|range| scoper.scope_raw(&input[range]))
            .flat_map(HashMap::into_keys)
            .collect();
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

    let mut global = 0..0;
    for (i, line) in input.split_inclusive(['\n', '\r']).enumerate() {
        let i = i + 1;

        global.end = global.start + line.len();

        let mut highlights = Vec::new();

        while let Some(Range { start, end }) = range {
            assert!(
                start >= global.start,
                "Previous iteration sliced incorrectly"
            );

            if start >= global.end {
                // Ranges are sorted and we're beyond this line's range; go next line.
                break;
            }

            assert!(global.contains(&start));

            if end <= global.end {
                // Range is fully contained in current line; push and go next range.
                highlights.push(relative_to(global.start, start..end));
                range = ranges.next();
                continue;
            }

            // There's a split; split the current `range` up over lines
            assert!(!global.contains(&end));
            highlights.push(relative_to(global.start, start..global.end));
            range = Some(global.end..end);
            break;
        }

        assert!(
            match lines.last() {
                Some(Line { number, .. }) => number + 1 == i,
                None => true,
            },
            "Non-consecutive lines pushed: '{}' after '{:?}'",
            line.escape_debug(),
            lines.last(),
        );

        lines.push(Line {
            number: i,
            content: line,
            highlights: if highlights.is_empty() {
                None
            } else {
                assert!(
                    highlights.iter().all(|h| line.get(h.clone()).is_some()),
                    "Highlights range '{highlights:?}' out of range for line '{}'",
                    line.escape_debug()
                );

                Some(Ranges::from_iter(highlights))
            },
        });

        global.start = global.end;
    }

    lines
}

#[cfg(test)]
#[allow(clippy::single_range_in_vec_init)]
mod tests {
    use super::*;
    use crate::scoping::{regex::Regex, Scoper};
    use crate::RegexPattern;
    use rstest::rstest;

    fn make_regex(pattern: &str) -> Box<dyn Scoper> {
        Box::new(Regex::new(RegexPattern::new(pattern).unwrap()))
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
                highlights: None
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
                highlights: None
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
                highlights: make_highlights([0..5])
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
                highlights: make_highlights([2..5])
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
                highlights: make_highlights([3..5])
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
                highlights: None
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
                highlights: make_highlights([0..5])
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
                highlights: None
            },
            Line {
                number: 2,
                content: "world",
                highlights: None
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
                highlights: make_highlights([0..5])
            },
            Line {
                number: 2,
                content: "world",
                highlights: None
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
                highlights: make_highlights([0..5])
            },
            Line {
                number: 2,
                content: "world",
                highlights: make_highlights([0..5])
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
                highlights: make_highlights([0..5])
            },
            Line {
                number: 2,
                content: "\n",
                highlights: None
            },
            Line {
                number: 3,
                content: "\n",
                highlights: None
            },
            Line {
                number: 4,
                content: "world",
                highlights: make_highlights([0..5])
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
                highlights: None
            },
            Line {
                number: 2,
                content: "\n",
                highlights: None
            },
            Line {
                number: 3,
                content: "\n",
                highlights: None
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
                highlights: make_highlights([0..6]) // ⚠️ newline included in range
            },
            Line {
                number: 2,
                content: "world",
                highlights: make_highlights([0..5])
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
                highlights: make_highlights([3..6]) // ⚠️ newline included in range
            },
            Line {
                number: 2,
                content: "world",
                highlights: make_highlights([0..2])
            },
        ],
    )]
    fn test_grep(
        #[case] input: &str,
        #[case] scopers: Vec<Box<dyn Scoper>>,
        #[case] expected: Vec<Line>,
    ) {
        assert_eq!(grep(input, &scopers), expected);
    }
}
