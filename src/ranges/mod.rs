use itertools::Itertools;
use log::{debug, trace};
use std::{
    fmt::Debug,
    ops::{Range, Sub},
    slice::{Iter, IterMut},
};

/// A collection of [`Range`]s.
///
/// This type implements a couple utility functions to work with collections of ranges.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ranges<Idx: Ord + Copy + Debug> {
    inner: Vec<Range<Idx>>,
}

impl<Idx: Ord + Copy + Debug> Ranges<Idx> {
    /// Returns the number of ranges.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns whether the collection is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub(crate) fn merge(&mut self) -> &mut Self {
        debug_assert!(self.is_sorted(), "Merging relies on sorted ranges");

        debug!("Merging ranges: {:?}", self);

        // We're potentially removing elements, so building up from a new allocation is
        // easiest.
        let capacity = self.len();
        let mut res = Vec::with_capacity(capacity);

        let mut previous: Option<Range<Idx>> = None;
        for current in {
            // Reborrow, but can drop mutability for the iteration.
            &*self
        } {
            match previous {
                Some(prev_range) => {
                    let overlaps = prev_range.end > current.start;
                    let borders = prev_range.end == current.start;
                    if overlaps || borders {
                        let start = prev_range.start;
                        let end = prev_range.end.max(current.end);

                        // Build it up. Don't push yet: there might be an unknown number
                        // of elements more to merge.
                        previous = Some(start..end);
                    } else {
                        res.push(prev_range);
                        previous = Some(current.to_owned());
                    }
                }
                None => previous = Some(current.to_owned()),
            }
        }

        if let Some(prev_range) = previous {
            // Potentially dangling element.
            res.push(prev_range);
        }

        assert!(
            // Cheap enough to do at runtime.
            res.len() <= capacity,
            "Merging should not increase the number of ranges"
        );

        self.inner = res;
        self.inner.shrink_to_fit(); // Might have removed elements, so yield memory back
        debug!("Merged ranges: {:?}", self);
        self
    }

    fn is_sorted(&self) -> bool {
        self.inner.windows(2).all(|w| w[0].start <= w[1].start)
    }

    /// Returns an iterator over the ranges, yielding references.
    pub fn iter(&self) -> Iter<'_, Range<Idx>> {
        self.into_iter()
    }

    /// Returns an iterator over the ranges, yielding mutable references.
    pub fn iter_mut(&mut self) -> IterMut<'_, Range<Idx>> {
        self.into_iter()
    }
}

impl<Idx: Ord + Copy + Debug> IntoIterator for Ranges<Idx> {
    type Item = Range<Idx>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, Idx: Ord + Copy + Debug> IntoIterator for &'a Ranges<Idx> {
    type Item = &'a Range<Idx>;
    type IntoIter = Iter<'a, Range<Idx>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a, Idx: Ord + Copy + Debug> IntoIterator for &'a mut Ranges<Idx> {
    type Item = &'a mut Range<Idx>;
    type IntoIter = IterMut<'a, Range<Idx>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

impl<Idx: Ord + Copy + Debug> FromIterator<Range<Idx>> for Ranges<Idx> {
    fn from_iter<I: IntoIterator<Item = Range<Idx>>>(iter: I) -> Self {
        let mut inner = iter.into_iter().collect_vec();
        inner.sort_by_key(|r| r.start);

        Self { inner }
    }
}

impl<Idx: Ord + Copy + Debug> Sub for Ranges<Idx> {
    type Output = Self;

    /// Subtract the `rhs` from `self`, such that all ranges in `rhs` are removed from
    /// `self`.
    ///
    /// ## Examples
    ///
    /// Using single ranges for simplicity.
    ///
    /// ### To the left, no overlap
    ///
    /// ```text
    /// self:        |-------XXXXX--->
    /// rhs:         |-XXXX---------->
    /// self - rhs = |-------XXXXX--->
    /// ```
    ///
    /// ```rust
    /// use srgn::ranges::Ranges as Rs;
    ///
    /// let self_: Rs<i8> = vec![4..5].into_iter().collect();
    /// let rhs: Rs<i8> = vec![1..2].into_iter().collect();
    ///
    /// assert_eq!(self_ - rhs, vec![4..5].into_iter().collect());
    /// ```
    ///
    /// ### Regular overlap
    ///
    /// ```text
    /// self:        |---XXXXX------->
    /// rhs:         |-XXXX---------->
    /// self - rhs = |-----XXX------->
    /// ```
    ///
    /// ```rust
    /// use srgn::ranges::Ranges as Rs;
    ///
    /// let self_: Rs<i8> = vec![3..6].into_iter().collect();
    /// let rhs: Rs<i8> = vec![1..4].into_iter().collect();
    ///
    /// assert_eq!(self_ - rhs, vec![4..6].into_iter().collect());
    /// ```
    ///
    /// ### Splits
    ///
    /// ```text
    /// self:        |--XXXXXXXXXX--->
    /// rhs:         |----XXXXX------>
    /// self - rhs = |--XX-----XXX--->
    /// ```
    ///
    /// ```rust
    /// use srgn::ranges::Ranges as Rs;
    ///
    /// let self_: Rs<i8> = vec![1..6].into_iter().collect();
    /// let rhs: Rs<i8> = vec![3..5].into_iter().collect();
    ///
    /// assert_eq!(self_ - rhs, vec![1..3, 5..6].into_iter().collect());
    /// ```
    ///
    /// ### Envelops
    ///
    /// ```text
    /// self:        |----XXXXX------>
    /// rhs:         |--XXXXXXXXXX--->
    /// self - rhs = |--------------->
    /// ```
    ///
    /// ```rust
    /// use srgn::ranges::Ranges as Rs;
    ///
    /// let self_: Rs<i8> = vec![3..5].into_iter().collect();
    /// let rhs: Rs<i8> = vec![1..6].into_iter().collect();
    ///
    /// assert_eq!(self_ - rhs, vec![].into_iter().collect());
    /// ```
    ///
    /// ### Combined
    ///
    /// ```rust
    /// use srgn::ranges::Ranges as Rs;
    ///
    /// let self_: Rs<u16> = vec![2..7, 10..15, 20..25, 40..50, 100..137, 200..300].into_iter().collect();
    /// let rhs: Rs<u16> = vec![0..1, 5..9, 12..15, 20..23, 30..35, 40..50, 99..138].into_iter().collect();
    ///
    /// assert_eq!(self_ - rhs, vec![2..5, 10..12, 23..25, 200..300].into_iter().collect());
    /// ```
    ///
    /// ## Complexity
    ///
    /// Complexity is linear with `O(len(self) + len(rhs))`.
    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(self.is_sorted() && rhs.is_sorted());

        // We are potentially splitting single items into two, or removing them
        // entirely. Hence, a new allocation is easiest.
        let mut result = Vec::with_capacity(self.inner.len());

        let mut left_iter = self.into_iter();
        let mut right_iter = rhs.into_iter();

        let mut left = left_iter.next();
        let mut right = right_iter.next();

        loop {
            trace!("Subtracting, left: {:?}, right: {:?}", left, right);
            match (&mut left, &mut right) {
                (None, _) => break,
                (Some(l), None) => {
                    result.push(l.to_owned());
                    left = left_iter.next();
                }
                (Some(l), Some(r)) => {
                    let right_entirely_left_of_left = r.end <= l.start;
                    if right_entirely_left_of_left {
                        // Advancing `right` until there's overlap.
                        right = right_iter.next();
                        continue;
                    }

                    let right_entirely_right_of_left = r.start >= l.end;
                    if right_entirely_right_of_left {
                        // Advancing `left` until there's overlap.
                        result.push(l.to_owned());
                        left = left_iter.next();
                        continue;
                    }

                    if r.start > l.start {
                        // A small part to the left is 'free', aka uncovered by `right`;
                        // any later `right` will be *even further* right, so we can
                        // safely push this part.
                        result.push(l.start..r.start);
                    }

                    // We 'decimated' this one, so adjust its size.
                    l.start = r.end;

                    let entire_rest_of_left_is_covered = r.end >= l.end;
                    if entire_rest_of_left_is_covered {
                        // This one is unrecoverable no matter what comes next, so skip ahead.
                        left = left_iter.next();
                        continue;
                    }

                    // The `right` one did all mutation/cutting down to `left` it can,
                    // check out the next one.
                    right = right_iter.next();
                }
            }
        }

        result.shrink_to_fit(); // Might have removed elements, so yield memory back
        result.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

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
        // ⚠️
        vec![0..0],
        vec![0..0],
        vec![0..0]
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
    fn test_ranges_subtraction<I: IntoIterator<Item = Range<usize>>>(
        #[case] left: I,
        #[case] right: I,
        #[case] expected: I,
    ) {
        let left = Ranges::from_iter(left);
        let right = Ranges::from_iter(right);
        let res = left - right;

        assert_eq!(res, Ranges::from_iter(expected));
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
    fn test_merge<I: IntoIterator<Item = Range<isize>>>(#[case] ranges: I, #[case] expected: I) {
        // ⚠️ Do not go through constructor etc., so we can test merging in isolation.
        let mut ranges = Ranges {
            inner: ranges.into_iter().collect(),
        };

        ranges.merge();

        assert_eq!(ranges, expected.into_iter().collect());
    }

    #[rstest]
    #[case(-1i8..2)]
    #[case(-1i16..2)]
    #[case(-1i32..2)]
    #[case(-1i64..2)]
    #[case(-1i128..2)]
    #[case(-1isize..2)]
    #[case(1u8..2)]
    #[case(1u16..2)]
    #[case(1u32..2)]
    #[case(1u64..2)]
    #[case(1u128..2)]
    #[case(1usize..2)]
    fn test_various_generic_inputs(#[case] range: Range<impl Ord + Copy + Debug>) {
        let _ = Ranges::from_iter(vec![range]);
    }

    #[test]
    fn test_iteration() {
        let ranges: Ranges<usize> = Ranges::default();

        // By reference, implicitly
        for _range in &ranges {
            //
        }

        // By value, implicitly
        for _range in ranges {
            //
        }

        // Moved out
    }
}
