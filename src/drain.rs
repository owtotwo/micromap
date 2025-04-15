// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use super::Map;
use core::{iter::FusedIterator, mem::MaybeUninit};

impl<K, V, const N: usize> Map<K, V, N> {
    /// Clears the map, returning all key-value pairs as an iterator. For reuse, the
    /// memory of capacity `N` will be keeped.
    ///
    /// If the returned iterator is dropped before being fully consumed, it drops the
    /// remaining key-value pairs. The returned iterator keeps a mutable borrow on
    /// the map to optimize its implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use micromap::Map;
    ///
    /// let mut a = Map::<_, _, 3>::new();
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    ///
    /// for (k, v) in a.drain().take(1) {
    ///     assert!(k == 1 || k == 2);
    ///     assert!(v == "a" || v == "b");
    /// }
    ///
    /// assert!(a.is_empty());
    /// ```
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        let drain = Drain {
            iter: self.pairs[0..self.len].iter_mut(),
        };
        self.len = 0;
        drain
    }
}

/// A draining iterator over the entries of a `Map`.
///
/// This `struct` is created by the [`drain`][`Map::drain`] method on [`Map`].
/// See its documentation for more.
///
/// # Example
///
/// ```
/// use micromap::Map;
///
/// let mut map = Map::from([
///     ("a", 1),
///     ("b", 2),
/// ]);
/// assert_eq!(map.len(), 2);
///
/// let iter = map.drain();
/// assert_eq!(iter.len(), 2);
/// // assert_eq!(map.len(), 0); // Drain keeps a mutable borrow on the map
/// drop(iter);
/// assert_eq!(map.len(), 0); // Now we can use the map's borrow
/// ```
pub struct Drain<'a, K, V> {
    iter: core::slice::IterMut<'a, MaybeUninit<(K, V)>>,
}

impl<K, V> Drop for Drain<'_, K, V> {
    fn drop(&mut self) {
        for pair in &mut self.iter {
            unsafe { pair.assume_init_drop() };
        }
    }
}

impl<K, V> Iterator for Drain<'_, K, V> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|p| unsafe { p.assume_init_read() })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.iter.len(), Some(self.iter.len()))
    }
}

impl<K, V> ExactSizeIterator for Drain<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> FusedIterator for Drain<'_, K, V> {}

#[cfg(test)]
mod tests {
    use crate::Map;

    #[test]
    fn normal_drain() {
        let mut map = Map::<char, u8, 10>::from_iter([('a', 97), ('b', 98), ('c', 99), ('d', 100)]);
        let mut cloned_map = map.clone();

        let mut drain = map.drain();

        // For ExactSizeIterator
        assert_eq!(drain.len(), drain.size_hint().0);

        // Consume the first two items by iterator
        assert_eq!(drain.next(), Some(('a', 97)));
        assert_eq!(drain.next(), Some(('b', 98)));

        // We can fuse the drain
        let mut fuse_it = drain.fuse();
        assert_eq!(fuse_it.next(), Some(('c', 99)));
        assert_eq!(fuse_it.next(), Some(('d', 100)));

        // Further calls to next() should return None
        assert!(fuse_it.next().is_none());
        // Then fuse works. (It doesn't make sense in our Drain really, but it can.)
        assert!(fuse_it.next().is_none());

        let mut drain = cloned_map.drain();
        assert_eq!(drain.next(), Some(('a', 97)));
        // Three elements left for Drop
        drop(drain);
    }
}
