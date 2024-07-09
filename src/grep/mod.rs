/// Module for (very leaky) abstractions over [`std::ops::Range`] to model global and
/// local ranges, for increased type safety in handling (instead of raw `usize`s).
pub mod ranges {
    use std::ops::{Range, Sub};

    /// A range modelled after [`Range`], where [`GlobalRange::start`] and
    /// [`GlobalRange::end`] denote *global* positions, across the entire input.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
    pub struct GlobalRange<T = usize> {
        /// Like [`std::ops::Range::start`].
        pub start: T,
        /// Like [`std::ops::Range::end`].
        pub end: T,
    }

    impl GlobalRange {
        /// Like [`std::ops::Range::is_empty`].
        #[must_use]
        pub fn is_empty(&self) -> bool {
            Range::from(*self).is_empty()
        }

        /// Like [`std::ops::Range::contains`].
        #[must_use]
        pub fn contains(&self, item: usize) -> bool {
            Range::from(*self).contains(&item)
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
}
