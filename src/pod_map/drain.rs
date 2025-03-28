// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use bytemuck::Pod;

use super::Drain;
use core::iter::FusedIterator;

impl<K, V> Drop for Drain<'_, K, V> {
    fn drop(&mut self) {
        for pair in &mut self.values {
            unsafe { pair.assume_init_drop() };
        }
    }
}

impl<K: PartialEq + Pod, V> Iterator for Drain<'_, K, V> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let k = *self.keys.next()?;
        let v = self
            .values
            .next()
            .map(|p| unsafe { p.assume_init_read() })?;
        Some((k, v))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.keys.len(), Some(self.keys.len()))
    }
}

impl<K: PartialEq + Pod, V> ExactSizeIterator for Drain<'_, K, V> {
    #[inline]
    fn len(&self) -> usize {
        self.keys.len()
    }
}

impl<K: PartialEq + Pod, V> FusedIterator for Drain<'_, K, V> {}

#[cfg(test)]
mod tests {
    use crate::PodMap;

    #[test]
    fn normal_drain() {
        let mut pod_map =
            PodMap::<u8, u8, 10>::from_iter([(b'a', 97), (b'b', 98), (b'c', 99), (b'd', 100)]);
        let mut cloned_map = pod_map.clone();

        let mut drain = pod_map.drain();

        // For ExactSizeIterator
        assert_eq!(drain.len(), drain.size_hint().0);

        // Consume the first two items by iterator
        assert_eq!(drain.next(), Some((b'a', 97)));
        assert_eq!(drain.next(), Some((b'b', 98)));

        // We can fuse the drain
        let mut fuse_it = drain.fuse();
        assert_eq!(fuse_it.next(), Some((b'c', 99)));
        assert_eq!(fuse_it.next(), Some((b'd', 100)));

        // Further calls to next() should return None
        assert!(fuse_it.next().is_none());
        // Then fuse works. (It doesn't make sense in our Drain really, but it can.)
        assert!(fuse_it.next().is_none());

        let mut drain = cloned_map.drain();
        assert_eq!(drain.next(), Some((b'a', 97)));
        // Three elements left for Drop
        drop(drain);
    }
}
