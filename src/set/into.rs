// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-FileCopyrightText: Copyright (c) 2025 owtotwo
// SPDX-License-Identifier: MIT

use crate::Set;
use core::mem::MaybeUninit;

impl<T: PartialEq, const N: usize> Set<T, N> {
    /// Consumes itself and generates a new set with the same data
    /// as `self` by [`Copy`], but with **different capacity**.
    /// So it's a fast way to create a Set of one capacity from
    /// another capacity. If Element Type is not `Copy` but `Clone`,
    /// then [`clone_into`] is an alternative method.
    ///
    /// [`clone_into`]: Set::clone_into
    ///
    /// # Panics
    ///
    /// Panics if the length of the current set exceeds the capacity
    /// of the new set.
    #[inline]
    #[must_use]
    pub fn copy_into<const M: usize>(self) -> Set<T, M>
    where
        MaybeUninit<(T, ())>: Copy,
    {
        assert!(
            self.len() <= M,
            "The new Set<_, M> is too small to hold the data."
        );
        Set {
            map: self.map.copy_into(),
        }
    }

    /// Consumes itself and generates a new set with the same data
    /// as `self` by [`Clone`], but with **different capacity**.
    /// So it's a fast way to create a Set of one capacity from
    /// another capacity. If Element Type is `Copy`, then
    /// [`copy_into`] is preferred.
    ///
    /// [`copy_into`]: Set::copy_into
    ///
    /// # Panics
    ///
    /// Panics if the length of the current set exceeds the capacity
    /// of the new set.
    #[inline]
    #[must_use]
    pub fn clone_into<const M: usize>(self) -> Set<T, M>
    where
        MaybeUninit<(T, ())>: Clone,
    {
        assert!(
            self.len() <= M,
            "The new Set<_, M> is too small to hold the data."
        );
        Set {
            map: self.map.clone_into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_into() {
        let set: Set<i32, 3> = Set::from_iter([1, 2]);
        let new_capacity_set: Set<i32, 5> = set.copy_into();
        assert_eq!(new_capacity_set.len(), 2);
        assert!(new_capacity_set.contains(&1));
        assert!(new_capacity_set.contains(&2));
        assert!(!new_capacity_set.contains(&3));
    }

    #[test]
    #[should_panic(expected = "The new Set<_, M> is too small to hold the data.")]
    fn test_copy_into_panic() {
        let set: Set<i32, 3> = Set::from_iter([1, 2, 3]);
        let _small_set: Set<i32, 2> = set.copy_into();
    }

    #[test]
    fn test_clone_into() {
        let set: Set<i32, 3> = Set::from_iter([1, 2]);
        let larger_set: Set<i32, 5> = set.clone_into();
        assert_eq!(larger_set.len(), 2);
        assert!(larger_set.contains(&1));
        assert!(larger_set.contains(&2));
        assert!(!larger_set.contains(&3));
    }

    #[test]
    #[should_panic(expected = "The new Set<_, M> is too small to hold the data.")]
    fn test_clone_into_panic() {
        let set: Set<i32, 3> = Set::from_iter([1, 2, 3]);
        let _small_set: Set<i32, 1> = set.clone_into();
    }
}
