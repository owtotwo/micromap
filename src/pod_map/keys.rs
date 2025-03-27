// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use bytemuck::Pod;

use super::{IntoKeys, Keys, PodMap};
use core::iter::FusedIterator;

impl<K: PartialEq + Pod, V, const N: usize> PodMap<K, V, N> {
    /// An iterator visiting all keys in arbitrary order.
    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys { iter: self.iter() }
    }

    /// Consuming iterator visiting all keys in arbitrary order.
    #[inline]
    pub fn into_keys(self) -> IntoKeys<K, V, N> {
        IntoKeys {
            iter: self.into_iter(),
        }
    }
}

impl<K, V> Clone for Keys<'_, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Keys {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|p| p.0)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K: PartialEq + Pod, V, const N: usize> Iterator for IntoKeys<K, V, N> {
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<K> {
        self.iter.next().map(|p| p.0)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K, V> ExactSizeIterator for Keys<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K: PartialEq + Pod, V, const N: usize> ExactSizeIterator for IntoKeys<K, V, N> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> FusedIterator for Keys<'_, K, V> {}

impl<K: PartialEq + Pod, V, const N: usize> FusedIterator for IntoKeys<K, V, N> {}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn iterate_keys() {
        let mut m: PodMap<[u8; 3], i32, 10> = PodMap::new();
        m.insert(b"foo".to_owned(), 0);
        m.insert(b"bar".to_owned(), 0);
        let keys = m.keys();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys.collect::<Vec<_>>(), [b"foo", b"bar"]);
    }

    #[test]
    fn iterate_into_keys() {
        let mut m: PodMap<[u8; 3], i32, 10> = PodMap::new();
        m.insert(b"foo".to_owned(), 0);
        m.insert(b"bar".to_owned(), 0);
        let keys = m.into_keys();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys.collect::<Vec<_>>(), [b"bar".to_owned(), b"foo".to_owned()]);
    }
}
