// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-FileCopyrightText: Copyright (c) 2025 owtotwo
// SPDX-License-Identifier: MIT

use bytemuck::Pod;

use super::PodMap;
use core::mem::MaybeUninit;

impl<K: PartialEq + Pod, V, const N: usize> PodMap<K, V, N> {
    /// Consumes itself and generates a new one then swap the data
    /// to it, the new [`PodMap`] which with **different capacity**.
    /// So it's a fast way to create a [`PodMap`] of one capacity from
    /// another capacity.
    ///
    /// # Panics
    ///
    /// Panics if the length of the current PodMap exceeds the capacity
    /// of the new PodMap.
    #[inline]
    #[must_use]
    pub fn into<const M: usize>(mut self) -> PodMap<K, V, M> {
        assert!(
            self.len <= M,
            "The new PodMap<_, _, M> is too small to hold the data."
        );
        let mut other = PodMap {
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
        let pod_map: PodMap<i32, i32, 3> = PodMap::from_iter([(1, 10), (2, 20)]);
        let new_capacity_map: PodMap<i32, i32, 5> = pod_map.into();
        assert_eq!(new_capacity_map.len(), 2);
        assert_eq!(new_capacity_map.get(&1), Some(&10));
        assert_eq!(new_capacity_map.get(&2), Some(&20));
        assert_eq!(new_capacity_map.get(&3), None);
    }

    #[test]
    #[should_panic(expected = "The new PodMap<_, _, M> is too small to hold the data.")]
    fn test_into_panic() {
        let pod_map: PodMap<i32, i32, 3> = PodMap::from_iter([(1, 10), (2, 20), (3, 30)]);
        let _small_map: PodMap<i32, i32, 2> = pod_map.into();
    }
}
