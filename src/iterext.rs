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
