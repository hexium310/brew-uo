pub(crate) trait Range<T>: std::ops::Deref<Target = [T]> {
    fn range<R>(&self, range: &R) -> Option<std::slice::Iter<T>>
    where
        R: std::ops::RangeBounds<usize>,
    {
        let len = self.len();
        let start = match range.start_bound() {
            std::ops::Bound::Included(&n) => n,
            std::ops::Bound::Excluded(&n) => n + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&n) => n + 1,
            std::ops::Bound::Excluded(&n) => n,
            std::ops::Bound::Unbounded => len,
        };

        self.get(start..end).map(|v| v.iter())
    }
}

impl<T> Range<T> for &[T] {}

impl<T> Range<T> for Vec<T> {}
