// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-FileCopyrightText: Copyright (c) 2025 owtotwo
// SPDX-License-Identifier: MIT

use crate::Map;
use core::mem::MaybeUninit;

impl<K: PartialEq, V, const N: usize> Map<K, V, N> {
    /// Consumes itself and generates a new one then swap the data
    /// to it, the new [`Map`] which with **different capacity**.
    /// So it's a fast way to create a [`Map`] of one capacity from
    /// another capacity.
    ///
    /// # Panics
    ///
    /// Panics if the length of the current map exceeds the capacity
    /// of the new map.
    #[inline]
    #[must_use]
    pub fn into<const M: usize>(mut self) -> Map<K, V, M> {
        assert!(
            self.len <= M,
            "The new Map<_, _, M> is too small to hold the data."
        );
        let mut other = Map {
            len: self.len,
            pairs: [const { MaybeUninit::uninit() }; M],
        };
        other.pairs[..self.len].swap_with_slice(self.pairs[..self.len].as_mut());
        other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into() {
        let map: Map<i32, i32, 3> = Map::from_iter([(1, 10), (2, 20)]);
        let new_capacity_map: Map<i32, i32, 5> = map.into();
        assert_eq!(new_capacity_map.len(), 2);
        assert_eq!(new_capacity_map.get(&1), Some(&10));
        assert_eq!(new_capacity_map.get(&2), Some(&20));
        assert_eq!(new_capacity_map.get(&3), None);
    }

    #[test]
    #[should_panic(expected = "The new Map<_, _, M> is too small to hold the data.")]
    fn test_into_panic() {
        let map: Map<i32, i32, 3> = Map::from_iter([(1, 10), (2, 20), (3, 30)]);
        let _small_map: Map<i32, i32, 2> = map.into();
    }
}
