// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::{IntoValues, Map, Values, ValuesMut};
use core::iter::FusedIterator;

impl<K: PartialEq, V, const N: usize> Map<K, V, N> {
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

impl<K: PartialEq, V, const N: usize> Iterator for IntoValues<K, V, N> {
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

impl<'a, K, V> ExactSizeIterator for Values<'a, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, K, V> ExactSizeIterator for ValuesMut<'a, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K: PartialEq, V, const N: usize> ExactSizeIterator for IntoValues<K, V, N> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, K, V> FusedIterator for Values<'a, K, V> {}

impl<'a, K, V> FusedIterator for ValuesMut<'a, K, V> {}

impl<K: PartialEq, V, const N: usize> FusedIterator for IntoValues<K, V, N> {}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn iterate_values() {
        let mut m: Map<String, i32, 10> = Map::new();
        m.insert("one".to_string(), 42);
        m.insert("two".to_string(), 16);
        assert_eq!(58, m.values().sum());
    }

    #[test]
    fn iterate_values_mut() {
        let mut m: Map<String, i32, 10> = Map::new();
        m.insert("one".to_string(), 42);
        m.insert("two".to_string(), 16);
        m.values_mut().for_each(|v| *v *= 2);
        assert_eq!(116, m.values().sum());
    }

    #[test]
    fn iterate_values_with_blanks() {
        let mut m: Map<String, i32, 10> = Map::new();
        m.insert("one".to_string(), 1);
        m.insert("two".to_string(), 3);
        m.insert("three".to_string(), 5);
        m.remove("two");
        assert_eq!(m.values().collect::<Vec<_>>(), [&1, &5]);
    }

    #[test]
    fn into_values_drop() {
        use std::rc::Rc;
        let mut m: Map<i32, Rc<()>, 8> = Map::new();
        let v = Rc::new(());
        for i in 0..8 {
            m.insert(i, Rc::clone(&v));
        }
        assert_eq!(9, Rc::strong_count(&v));
        m.into_values();
        assert_eq!(1, Rc::strong_count(&v));
    }
}
