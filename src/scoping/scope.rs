use crate::scoping::scope::Scope::{In, Out};
use itertools::Itertools;
use log::{debug, trace};
use std::{borrow::Cow, ops::Range};

/// Indicates whether a given string part is in scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope<'viewee, T> {
    /// The given part is in scope for processing.
    In(T),
    /// The given part is out of scope for processing.
    ///
    /// Treated as immutable, view-only.
    Out(&'viewee str),
}

/// A read-only scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ROScope<'viewee>(pub Scope<'viewee, &'viewee str>);

/// Multiple read-only scopes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ROScopes<'viewee>(pub Vec<ROScope<'viewee>>);

/// A read-write scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RWScope<'viewee>(pub Scope<'viewee, Cow<'viewee, str>>);

/// Multiple read-write scopes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RWScopes<'viewee>(pub Vec<RWScope<'viewee>>);

impl<'viewee> ROScope<'viewee> {
    /// Check whether the scope is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let s: &str = self.into();
        s.is_empty()
    }
}

impl<'viewee> ROScopes<'viewee> {
    /// Construct a new instance from the given raw ranges.
    ///
    /// The passed `input` will be traversed according to `ranges`: all specified
    /// `ranges` are taken as [`In`] scope, everything not covered by a range is [`Out`]
    /// of scope.
    ///
    /// ## Panics
    ///
    /// Panics if the given `ranges` contain indices out-of-bounds for `input`.
    #[must_use]
    pub fn from_raw_ranges(input: &'viewee str, ranges: Vec<Range<usize>>) -> Self {
        trace!("Constructing scopes from raw ranges: {:?}", ranges);

        let mut scopes = Vec::with_capacity(ranges.len());

        let mut last_end = 0;
        for Range { start, end } in ranges.into_iter().sorted_by_key(|r| r.start) {
            scopes.push(ROScope(Out(&input[last_end..start])));
            scopes.push(ROScope(In(&input[start..end])));
            last_end = end;
        }

        if last_end < input.len() {
            scopes.push(ROScope(Out(&input[last_end..])));
        }

        scopes.retain(|s| !s.is_empty());

        debug!("Scopes: {:?}", scopes);

        ROScopes(scopes)
    }

    /// Inverts the scopes: what was previously [`In`] is now [`Out`], and vice versa.
    #[must_use]
    pub fn invert(self) -> Self {
        trace!("Inverting scopes: {:?}", self.0);
        let scopes = self
            .0
            .into_iter()
            .map(|s| match s {
                ROScope(In(s)) => ROScope(Out(s)),
                ROScope(Out(s)) => ROScope(In(s)),
            })
            .collect();
        trace!("Inverted scopes: {:?}", scopes);

        Self(scopes)
    }
}

/// Checks for equality, regarding only raw [`str`] parts, i.e. disregards whether an
/// element is [`In`] or [`Out`] of scope.
impl PartialEq<&str> for ROScopes<'_> {
    fn eq(&self, other: &&str) -> bool {
        let mut start = 0;
        let mut end = None;

        for scope in &self.0 {
            let s: &str = scope.into();
            end = Some(start + s.len());

            let Some(substring) = other.get(start..end.unwrap()) else {
                return false;
            };

            if substring != s {
                return false;
            }

            start = end.unwrap();
        }

        match end {
            Some(e) => other.len() == e,
            None => other.is_empty(),
        }
    }
}

impl PartialEq<ROScopes<'_>> for &str {
    fn eq(&self, other: &ROScopes<'_>) -> bool {
        other == self
    }
}

impl<'viewee> From<&'viewee ROScope<'viewee>> for &'viewee str {
    /// Get the underlying string slice.
    ///
    /// All variants contain such a slice, so this is a convenient method.
    fn from(s: &'viewee ROScope) -> Self {
        match s.0 {
            In(s) | Out(s) => s,
        }
    }
}

impl<'viewee> From<ROScope<'viewee>> for RWScope<'viewee> {
    fn from(s: ROScope<'viewee>) -> Self {
        match s.0 {
            In(s) => RWScope(In(Cow::Borrowed(s))),
            Out(s) => RWScope(Out(s)),
        }
    }
}

impl<'viewee> From<&'viewee RWScope<'viewee>> for &'viewee str {
    /// Get the underlying string slice.
    ///
    /// All variants contain such a slice, so this is a convenient method.
    fn from(s: &'viewee RWScope) -> Self {
        match &s.0 {
            In(s) => s,
            Out(s) => s,
        }
    }
}

/// Subtract the `right` from the `left`, such that all ranges in `right` are removed
/// from `left`.
///
/// ## Preconditions
///
/// Both inputs are...
///
/// - sorted by [`Range::start`],
/// - non-overlapping in each input.
///
/// ## Illustration
///
/// Using single ranges for simplicity.
///
/// ### To the left, no overlap
///
/// ```text
/// L: |-------XXXXX--->
/// R: |-XXXX---------->
/// =  |-------XXXXX--->
/// ```
///
/// ### Regular overlap
///
/// ```text
/// L: |---XXXXX------->
/// R: |-XXXX---------->
/// =  |-----XXX------->
/// ```
///
/// ### Splits
///
/// ```text
/// L: |--XXXXXXXXXX--->
/// R: |----XXXXX------>
/// =  |--XX-----XXX--->
/// ```
///
/// ### Envelops
///
/// ```text
/// L: |----XXXXX------>
/// R: |--XXXXXXXXXX--->
/// =  |--------------->
/// ```
pub(crate) fn subtract<T>(mut left: Vec<Range<T>>, right: &Vec<Range<T>>) -> Vec<Range<T>>
where
    T: Ord + Copy + std::fmt::Debug,
{
    let mut res = Vec::with_capacity(left.len());

    #[cfg(debug_assertions)]
    {
        let is_sorted =
            |ranges: &Vec<Range<T>>| ranges.windows(2).all(|w| w[0].start <= w[1].start);
        let is_not_overlapping =
            |ranges: &Vec<Range<T>>| ranges.windows(2).all(|w| w[0].end <= w[1].start);

        for ranges in &[&left, right] {
            trace!("Checking preconditions for ranges: {:?}", &ranges);
            debug_assert!(is_sorted(ranges));
            debug_assert!(is_not_overlapping(ranges));
        }
    }

    'outer: for l in &mut left {
        for r in right {
            if r.end <= l.start {
                // Creeping in "from the left"
                continue;
            }

            if r.start >= l.end {
                // Gone past relevant range, go next
                break;
            }

            if r.start > l.start {
                // A small part to the left is 'free', aka uncovered by `r`; any later
                // `r` will be *even further* right, so we can safely push this part.
                res.push(l.start..r.start);
            }

            l.start = r.end;

            let is_fully_covered = l.start >= r.start && l.end <= r.end;
            if is_fully_covered {
                // This one is unrecoverable no matter what comes next, so skip ahead.
                continue 'outer;
            }
        }

        if !l.is_empty() {
            // Might have been decimated from mutation so much that it's empty now.
            res.push(l.clone());
        }
    }

    res
}

/// Merges consecutive, overlapping ranges.
pub(crate) fn merge<T>(mut ranges: Vec<Range<T>>) -> Vec<Range<T>>
where
    T: Ord + Copy + std::fmt::Debug,
{
    debug!("Merging ranges: {:?}", ranges);
    let mut res = Vec::with_capacity(ranges.len());

    ranges.sort_by_key(|r| r.start);

    let mut previous: Option<Range<T>> = None;
    for current in ranges {
        match previous {
            Some(prev_range) => {
                let overlaps = prev_range.end > current.start;
                let borders = prev_range.end == current.start;
                if overlaps || borders {
                    let start = prev_range.start.min(current.start);
                    let end = prev_range.end.max(current.end);

                    // Build it up. Don't push yet: there might be an unknown number of
                    // elements more to merge.
                    previous = Some(start..end);
                } else {
                    res.push(prev_range);
                    previous = Some(current);
                }
            }
            None => {
                previous = Some(current);
            }
        }
    }

    if let Some(prev_range) = previous {
        res.push(prev_range);
    }

    debug!("Merged ranges: {:?}", res);
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    // Base cases
    #[case(ROScopes(vec![ROScope(In("abc"))]), "abc", true)]
    #[case(ROScopes(vec![ROScope(In("cba"))]), "cba", true)]
    #[case(ROScopes(vec![ROScope(In("🦀"))]), "🦀", true)]
    #[case(ROScopes(vec![ROScope(In("🦀"))]), "🤗", false)]
    //
    // Substring matching
    #[case(ROScopes(vec![ROScope(In("a")), ROScope(In("b"))]), "ab", true)]
    #[case(ROScopes(vec![ROScope(In("a")), ROScope(In("b")), ROScope(In("c"))]), "abc", true)]
    //
    #[case(ROScopes(vec![ROScope(In("a")), ROScope(In("b"))]), "ac", false)]
    #[case(ROScopes(vec![ROScope(In("a")), ROScope(In("b"))]), "a", false)]
    #[case(ROScopes(vec![ROScope(In("a")), ROScope(In("b"))]), "b", false)]
    #[case(ROScopes(vec![ROScope(In("a")), ROScope(In("b")), ROScope(In("c"))]), "acc", false)]
    //
    // Length mismatch
    #[case(ROScopes(vec![ROScope(In("abc"))]), "abcd", false)]
    #[case(ROScopes(vec![ROScope(In("abcd"))]), "abc", false)]
    //
    // Partial emptiness
    #[case(ROScopes(vec![ROScope(In("abc"))]), "", false)]
    #[case(ROScopes(vec![ROScope(In(""))]), "abc", false)]
    #[case(ROScopes(vec![ROScope(Out(""))]), "abc", false)]
    #[case(ROScopes(vec![ROScope(In("")), ROScope(Out(""))]), "abc", false)]
    //
    // Full emptiness
    #[case(ROScopes(vec![ROScope(In(""))]), "", true)]
    #[case(ROScopes(vec![ROScope(Out(""))]), "", true)]
    #[case(ROScopes(vec![ROScope(In("")), ROScope(Out(""))]), "", true)]
    //
    // Types of scope doesn't matter
    #[case(ROScopes(vec![ROScope(In("a"))]), "a", true)]
    #[case(ROScopes(vec![ROScope(Out("a"))]), "a", true)]
    fn test_scoped_view_str_equality(
        #[case] scopes: ROScopes<'_>,
        #[case] string: &str,
        #[case] equal: bool,
    ) {
        assert!((scopes == string) == equal);
    }

    #[rstest]
    // For a fixed-size `left` interval, watch as the `right` interval slides past,
    // covering all cases along the way. This is the mental model used to come up with
    // the algorithm implementation.
    #[case(
        // Fully to the left
        vec![2..7],
        vec![0..1],
        vec![2..7]
    )]
    #[case(
        // Fully to the left; touching
        vec![2..7],
        vec![0..2],
        vec![2..7]
    )]
    #[case(
        // Single-element overlap
        vec![2..7],
        vec![0..3],
        vec![3..7]
    )]
    #[case(
        // Multi-element overlap
        vec![2..7],
        vec![0..4],
        vec![4..7]
    )]
    #[case(
        // Full overlap on both sides; nukes `left`
        vec![2..7],
        vec![0..8],
        vec![]
    )]
    #[case(
        // Pull `start` of `right` to the right, retract `end` of `right` to the back a
        // bit. Initially, an exact overlap.
        vec![2..7],
        vec![2..7],
        vec![]
    )]
    #[case(
        // Pull `start` of `right` further right
        vec![2..7],
        vec![3..7],
        vec![2..3]
    )]
    #[case(
        // Pull `end` of `right` fully into `left`; **this splits `left`**!
        vec![2..7],
        vec![3..6],
        vec![2..3, 6..7]
    )]
    // Full "end-to-end" example. For more, see
    // https://stackoverflow.com/q/6462272/11477374
    #[case(
        vec![2..7, 10..15, 20..25, 40..50, 100..137, 200..300],
        vec![0..1, 5..9, 12..15, 20..23, 30..35, 40..50, 99..138],
        vec![2..5, 10..12, 23..25, 200..300]
    )]
    //
    // More random edge cases
    #[case(
        vec![0..0],
        vec![0..0],
        vec![] // 🤷
    )]
    #[case(
        vec![0..1],
        vec![0..1],
        vec![]
    )]
    #[case(
        vec![0..2],
        vec![0..2],
        vec![]
    )]
    #[case(
        vec![0..2],
        vec![],
        vec![0..2]
    )]
    #[case(
        vec![],
        vec![0..1],
        vec![]
    )]
    #[case(
        vec![0..2],
        vec![0..0],
        vec![0..2]
    )]
    #[case(
        vec![0..2],
        vec![0..1],
        vec![1..2]
    )]
    #[case(
        vec![0..2],
        vec![1..2],
        vec![0..1]
    )]
    #[case(
        vec![0..3],
        vec![0..2],
        vec![2..3]
    )]
    #[case(
        vec![0..3],
        vec![2..3],
        vec![0..2]
    )]
    #[case(
        vec![1..3, 4..5],
        vec![1..2, 2..3],
        vec![4..5]
    )]
    #[case(
        vec![1..3, 4..5],
        vec![0..7],
        vec![]
    )]
    #[case(
        vec![1..3, 4..5, 8..10],
        vec![0..7],
        vec![8..10]
    )]
    #[case(
        vec![1..3, 4..10],
        vec![0..7],
        vec![7..10]
    )]
    #[case(
        vec![0..4],
        vec![0..1, 1..2, 2..3, 3..4],
        vec![]
    )]
    #[case(
        vec![0..4],
        vec![0..1, 1..2, 3..4],
        vec![2..3]
    )]
    fn test_ranges_subtraction(
        #[case] left: Vec<Range<isize>>,
        #[case] right: Vec<Range<isize>>,
        #[case] expected: Vec<Range<isize>>,
    ) {
        let res = subtract(left, &right);
        assert_eq!(res, expected);
    }

    #[rstest]
    // Uninteresting base cases
    #[case(
        vec![],
        vec![]
    )]
    #[case(
        vec![0..0],
        vec![0..0]
    )]
    #[case(
        vec![0..1],
        vec![0..1]
    )]
    #[case(
        vec![0..2],
        vec![0..2]
    )]
    //
    // Actual merges
    #[case(
        // Borders
        vec![0..1, 1..2],
        vec![0..2]
    )]
    #[case(
        // Doesn't border
        vec![0..1, 2..3],
        vec![0..1, 2..3]
    )]
    #[case(
        vec![0..1, 2..3, 4..5],
        vec![0..1, 2..3, 4..5]
    )]
    #[case(
        // Overlaps into
        vec![0..4, 3..5],
        vec![0..5]
    )]
    #[case(
        // Goes over
        vec![0..7, 3..5],
        vec![0..7]
    )]
    #[case(
        vec![0..5, 2..3, 3..5],
        vec![0..5]
    )]
    //
    // Also sorts
    #[case(
        vec![2..3, 0..3],
        vec![0..3]
    )]
    //
    // Other random edge cases
    #[case(
        vec![0..2, 0..2],
        vec![0..2]
    )]
    #[case(
        vec![0..0, 0..2],
        vec![0..2]
    )]
    #[case(
        vec![0..2, 0..0],
        vec![0..2]
    )]
    fn test_range_merge(#[case] range: Vec<Range<isize>>, #[case] expected: Vec<Range<isize>>) {
        let res = merge(range);
        assert_eq!(res, expected);
    }
}
