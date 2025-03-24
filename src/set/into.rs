// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-FileCopyrightText: Copyright (c) 2025 owtotwo
// SPDX-License-Identifier: MIT

use crate::Set;

impl<T: PartialEq, const N: usize> Set<T, N> {
    /// Consumes itself and generates a new one then swap the data
    /// to it, the new [`Set`] which with **different capacity**.
    /// So it's a fast way to create a [`Set`] of one capacity from
    /// another capacity.
    ///
    /// # Panics
    ///
    /// Panics if the length of the current set exceeds the capacity
    /// of the new set.
    #[inline]
    #[must_use]
    pub fn into<const M: usize>(self) -> Set<T, M> {
        assert!(
            self.len() <= M,
            "The new Set<_, M> is too small to hold the data."
        );
        Set {
            map: self.map.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into() {
        let set: Set<i32, 3> = Set::from_iter([1, 2]);
        let new_capacity_set: Set<i32, 5> = set.into();
        assert_eq!(new_capacity_set.len(), 2);
        assert!(new_capacity_set.contains(&1));
        assert!(new_capacity_set.contains(&2));
        assert!(!new_capacity_set.contains(&3));
    }

    #[test]
    #[should_panic(expected = "The new Set<_, M> is too small to hold the data.")]
    fn test_into_panic() {
        let set: Set<i32, 3> = Set::from_iter([1, 2, 3]);
        let _small_set: Set<i32, 2> = set.into();
    }
}
