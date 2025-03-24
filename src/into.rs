// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-FileCopyrightText: Copyright (c) 2025 owtotwo
// SPDX-License-Identifier: MIT

use crate::Map;
use core::mem::MaybeUninit;

impl<K: PartialEq, V, const N: usize> Map<K, V, N> {
    /// Consumes itself and generates a new map with the same data
    /// as `self` by [`Copy`], but with **different capacity**.
    /// So it's a fast way to create a Map of one capacity from
    /// another capacity.  If Key and Value Types are not `Copy`
    /// but `Clone`, then [`clone_into`] is an alternative method.
    ///
    /// [`clone_into`]: Map::clone_into
    ///
    /// # Panics
    ///
    /// Panics if the length of the current map exceeds the capacity
    /// of the new map.
    #[inline]
    #[must_use]
    pub fn copy_into<const M: usize>(self) -> Map<K, V, M>
    where
        MaybeUninit<(K, V)>: Copy,
    {
        assert!(
            self.len <= M,
            "The new Map<_, _, M> is too small to hold the data."
        );
        let mut other = Map {
            len: self.len,
            pairs: [const { MaybeUninit::uninit() }; M],
        };
        other.pairs[..self.len].copy_from_slice(self.pairs[..self.len].as_ref());
        other
    }

    /// Consumes itself and generates a new map with the same data
    /// as `self` by [`Clone`], but with **different capacity**.
    /// So it's a fast way to create a Map of one capacity from
    /// another capacity. If Key and Value Types are `Copy`, then
    /// [`copy_into`] is preferred.
    ///
    /// [`copy_into`]: Map::copy_into
    ///
    /// # Panics
    ///
    /// Panics if the length of the current map exceeds the capacity
    /// of the new map.
    #[inline]
    #[must_use]
    pub fn clone_into<const M: usize>(self) -> Map<K, V, M>
    where
        MaybeUninit<(K, V)>: Clone,
    {
        assert!(
            self.len <= M,
            "The new Map<_, _, M> is too small to hold the data."
        );
        let mut other = Map {
            len: self.len,
            pairs: [const { MaybeUninit::uninit() }; M],
        };
        other.pairs[..self.len].clone_from_slice(self.pairs[..self.len].as_ref());
        other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_into() {
        let map: Map<i32, i32, 3> = Map::from_iter([(1, 10), (2, 20)]);
        let new_capacity_map: Map<i32, i32, 5> = map.copy_into();
        assert_eq!(new_capacity_map.len(), 2);
        assert_eq!(new_capacity_map.get(&1), Some(&10));
        assert_eq!(new_capacity_map.get(&2), Some(&20));
        assert_eq!(new_capacity_map.get(&3), None);
    }

    #[test]
    #[should_panic(expected = "The new Map<_, _, M> is too small to hold the data.")]
    fn test_copy_into_panic() {
        let map: Map<i32, i32, 3> = Map::from_iter([(1, 10), (2, 20), (3, 30)]);
        let _small_map: Map<i32, i32, 2> = map.copy_into();
    }

    #[test]
    fn test_clone_into() {
        let map: Map<i32, i32, 3> = Map::from_iter([(1, 10), (2, 20)]);
        let larger_map: Map<i32, i32, 5> = map.clone_into();
        assert_eq!(larger_map.len(), 2);
        assert_eq!(larger_map.get(&1), Some(&10));
        assert_eq!(larger_map.get(&2), Some(&20));
        assert_eq!(larger_map.get(&3), None);
    }

    #[test]
    #[should_panic(expected = "The new Map<_, _, M> is too small to hold the data.")]
    fn test_clone_into_panic() {
        let map: Map<i32, i32, 3> = Map::from_iter([(1, 10), (2, 20), (3, 30)]);
        let _small_map: Map<i32, i32, 1> = map.clone_into();
    }
}
