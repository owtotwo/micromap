// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use bytemuck::Pod;

use super::{IntoValues, PodMap, Values, ValuesMut};
use core::iter::FusedIterator;

impl<K: PartialEq + Pod, V, const N: usize> PodMap<K, V, N> {
    /// An iterator visiting all values in arbitrary order.
    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        Values { iter: self.iter() }
    }

    /// An iterator visiting all values mutably in arbitrary order.
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            iter: self.iter_mut(),
        }
    }

    /// Consuming iterator visiting all the values in arbitrary order.
    #[inline]
    pub fn into_values(self) -> IntoValues<K, V, N> {
        IntoValues {
            iter: self.into_iter(),
        }
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|p| p.1)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|p| p.1)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K: PartialEq + Pod, V, const N: usize> Iterator for IntoValues<K, V, N> {
    type Item = V;

    #[inline]
    fn next(&mut self) -> Option<V> {
        self.iter.next().map(|p| p.1)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K, V> ExactSizeIterator for Values<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> ExactSizeIterator for ValuesMut<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K: PartialEq + Pod, V, const N: usize> ExactSizeIterator for IntoValues<K, V, N> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> FusedIterator for Values<'_, K, V> {}
impl<K, V> FusedIterator for ValuesMut<'_, K, V> {}
impl<K: PartialEq + Pod, V, const N: usize> FusedIterator for IntoValues<K, V, N> {}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn iterate_values() {
        let mut m: PodMap<u8, i32, 10> = PodMap::new();
        m.insert(1, 42);
        m.insert(2, 16);
        let it = m.values();
        assert_eq!(it.len(), 2);
        assert_eq!(58, it.sum());
    }

    #[test]
    fn iterate_values_mut() {
        let mut m: PodMap<u8, i32, 10> = PodMap::new();
        m.insert(1, 42);
        m.insert(2, 16);
        let it_mut = m.values_mut();
        assert_eq!(it_mut.len(), 2);
        assert_eq!(it_mut.len(), it_mut.size_hint().0);
        assert_eq!(it_mut.len(), it_mut.size_hint().1.unwrap());
        it_mut.for_each(|v| *v *= 2);
        assert_eq!(116, m.values().sum());
    }

    #[test]
    fn iterate_values_with_blanks() {
        let mut m: PodMap<u8, i32, 10> = PodMap::new();
        m.insert(1, 1);
        m.insert(2, 3);
        m.insert(3, 5);
        m.remove(&2);
        assert_eq!(m.values().collect::<Vec<_>>(), [&1, &5]);
    }

    #[test]
    fn into_values_drop() {
        use std::rc::Rc;
        let mut m: PodMap<i32, Rc<()>, 8> = PodMap::new();
        let v = Rc::new(());
        for i in 0..8 {
            m.insert(i, Rc::clone(&v));
        }
        assert_eq!(9, Rc::strong_count(&v));
        let mut values = m.into_values();
        assert!(values.next().is_some());
        assert_eq!(values.len(), 7);
        assert!(values.next().is_some());
        assert_eq!(values.len(), values.size_hint().0);
        drop(values);
        assert_eq!(1, Rc::strong_count(&v));
    }
}
