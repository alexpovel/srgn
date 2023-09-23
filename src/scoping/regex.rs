use super::{ScopedViewBuildStep, ScopedViewBuilder};
use crate::RegexPattern;
use crate::GLOBAL_SCOPE;
use log::{debug, trace};
use std::ops::Range;

#[derive(Debug)]
pub struct Regex {
    pattern: RegexPattern,
}

impl Regex {
    #[must_use]
    pub fn new(pattern: RegexPattern) -> Self {
        Self { pattern }
    }
}

impl Default for Regex {
    fn default() -> Self {
        Self::new(RegexPattern::new(GLOBAL_SCOPE).unwrap())
    }
}

impl ScopedViewBuildStep for Regex {
    fn scope<'a>(&self, input: &'a str) -> ScopedViewBuilder<'a> {
        ScopedViewBuilder::new(input).explode_from_ranges(|s| {
            let has_capture_groups = self.pattern.captures_len() > 1;

            if !has_capture_groups {
                trace!(
                    "No capture groups in pattern '{}', short-circuiting",
                    s.escape_debug()
                );
                return self
                    .pattern
                    .find_iter(s)
                    .flatten()
                    .map(|m| m.range())
                    .collect();
            }

            trace!(
                "Pattern '{}' has capture groups, iterating over matches",
                s.escape_debug()
            );
            let mut ranges = Vec::new();
            for cap in self.pattern.captures_iter(s).flatten() {
                let mut it = cap.iter();

                let overall_match = it
                    .next()
                    // https://docs.rs/regex/1.9.5/regex/struct.SubCaptureMatches.html
                    .expect("Entered iterator of matches, but zeroth (whole) match missing")
                    .expect("First element guaranteed to be non-None (whole match)");
                trace!(
                    "Overall match: {:?} ({}..{})",
                    overall_match.as_str().escape_debug(),
                    overall_match.start(),
                    overall_match.end()
                );

                let mut subranges = Vec::new();
                for group in it.flatten() {
                    trace!(
                        "Group match: {:?} ({}..{})",
                        group.as_str().escape_debug(),
                        group.start(),
                        group.end()
                    );
                    subranges.push(group.range());
                }

                let mut last_end = overall_match.range().start;
                for subrange in subranges.into_iter().rev() {
                    ranges.push(Range {
                        start: last_end,
                        end: subrange.start,
                    });

                    ranges.extend(shatter(&subrange));

                    last_end = subrange.end;
                }
            }

            debug!("Ranges to scope after regex: {:?}", ranges);
            ranges
        })
    }
}

/// For a given [`Range`], shatters it into pieces of length 1, returning a [`Vec`] of
/// length equal to the length of the original range.
fn shatter(range: &Range<usize>) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();

    for i in range.start..range.end {
        ranges.push(Range {
            start: i,
            end: i + 1,
        });
    }

    ranges
}
