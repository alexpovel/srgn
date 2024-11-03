/// Extension trait that adds parallel zipping functionality to iterators over iterators.
pub trait ParallelZipExt: Iterator {
    /// Zips multiple iterators in parallel, such that the nth invocation yields a
    /// [`Vec`] of all nth items of the subiterators.
    fn parallel_zip(self) -> ParallelZip<Self::Item>
    where
        Self: Sized,
        Self::Item: Iterator;
}

/// An iterator similar to [`std::iter::zip`], but instead it zips over *multiple
/// iterators* in parallel, such that the nth invocation yields a [`Vec`] of all
/// nth items of its subiterators.
#[derive(Debug)]
pub struct ParallelZip<I>(Vec<I>);

impl<I: Iterator> Iterator for ParallelZip<I> {
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }

        self.0.iter_mut().map(Iterator::next).collect()
    }
}

// Implement the extension trait for any iterator whose items are themselves iterators
impl<T> ParallelZipExt for T
where
    T: Iterator,
    T::Item: Iterator,
{
    fn parallel_zip(self) -> ParallelZip<T::Item> {
        ParallelZip(self.collect())
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::empty_once(
        Vec::<Vec<i32>>::new(),
        Vec::new(),
    )]
    #[case::empty_twice(
        vec![
            vec![],
            vec![],
        ],
        vec![],
    )]
    #[case::zips_to_shortest(
        vec![
            vec![0, 1, 2],
            vec![3, 4],
        ],
        vec![
            vec![0, 3],
            vec![1, 4],
        ]
    )]
    #[case::base_case(
        vec![
            vec![1, 2],
            vec![3, 4],
            vec![5, 6]
        ],
        vec![
            vec![1, 3, 5],
            vec![2, 4, 6]
        ]
    )]
    #[case::transpose_horizontal(
        vec![
            vec![1, 2, 3],
        ],
        vec![
            vec![1],
            vec![2],
            vec![3],
        ]
    )]
    #[case::transpose_vertical(
        vec![
            vec![1],
            vec![2],
            vec![3]
        ],
        vec![
            vec![1, 2, 3]
        ]
    )]
    fn test_parallel_zip(#[case] input: Vec<Vec<i32>>, #[case] expected: Vec<Vec<i32>>) {
        let iters = input.into_iter().map(IntoIterator::into_iter).collect_vec();
        let res = iters.into_iter().parallel_zip().collect_vec();
        assert_eq!(res, expected);
    }
}
