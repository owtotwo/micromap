// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-FileCopyrightText: Copyright (c) 2025 owtotwo
// SPDX-License-Identifier: MIT

use crate::Set;
use core::ops::{BitAnd, BitOr, BitXor, Sub};

// // {sum::<M, N>()}
// pub const fn sum<const A: usize, const B: usize>() -> usize {
//     A + B
// }

pub const fn min<const A: usize, const B: usize>() -> usize {
    if A < B {
        A
    } else {
        B
    }
}

impl<T: PartialEq + Clone, const N: usize, const M: usize> BitOr<&Set<T, M>> for &Set<T, N>
where
    [(); M + N]:,
{
    type Output = Set<T, { M + N }>;

    /// Returns the union of `self` and `rhs` as a new `Set<T, N>`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use micromap::Set;
    ///
    /// let a = Set::from([1, 2, 3]);
    /// let b = Set::from([3, 4, 5]);
    ///
    /// let set = &a | &b;
    ///
    /// let mut i = 0;
    /// let expected = [1, 2, 3, 4, 5];
    /// for x in &set {
    ///     assert!(expected.contains(x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn bitor(self, rhs: &Set<T, M>) -> Self::Output {
        self.union(rhs).cloned().collect()
    }
}

impl<T: PartialEq + Clone, const N: usize, const M: usize> BitAnd<&Set<T, M>> for &Set<T, N>
where
    [(); min::<M, N>()]:,
{
    type Output = Set<T, { min::<M, N>() }>;

    /// Returns the intersection of `self` and `rhs` as a new `Set<T, N>`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use micromap::Set;
    ///
    /// let a = Set::from([1, 2, 3]);
    /// let b = Set::from([2, 3, 4]);
    ///
    /// let set = &a & &b;
    ///
    /// let mut i = 0;
    /// let expected = [2, 3];
    /// for x in &set {
    ///     assert!(expected.contains(x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn bitand(self, rhs: &Set<T, M>) -> Self::Output {
        self.intersection(rhs).cloned().collect()
    }
}

impl<T: PartialEq + Clone, const N: usize, const M: usize> BitXor<&Set<T, M>> for &Set<T, N>
where
    [(); M + N]:,
{
    type Output = Set<T, { M + N }>;

    /// Returns the symmetric difference of `self` and `rhs` as a new `Set<T, N>`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use micromap::Set;
    ///
    /// let a = Set::from([1, 2, 3]);
    /// let b = Set::from([3, 4, 5]);
    ///
    /// let set = &a ^ &b;
    ///
    /// let mut i = 0;
    /// let expected = [1, 2, 4, 5];
    /// for x in &set {
    ///     assert!(expected.contains(x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn bitxor(self, rhs: &Set<T, M>) -> Self::Output {
        self.symmetric_difference(rhs).cloned().collect()
    }
}

impl<T: PartialEq + Clone, const N: usize, const M: usize> Sub<&Set<T, M>> for &Set<T, N> {
    type Output = Set<T, N>;

    /// Returns the difference of `self` and `rhs` as a new `Set<T, N>`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use micromap::Set;
    ///
    /// let a = Set::from([1, 2, 3]);
    /// let b = Set::from([3, 4, 5]);
    ///
    /// let set = &a - &b;
    ///
    /// let mut i = 0;
    /// let expected = [1, 2];
    /// for x in &set {
    ///     assert!(expected.contains(x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn sub(self, rhs: &Set<T, M>) -> Self::Output {
        self.difference(rhs).cloned().collect()
    }
}

#[cfg(test)]
mod tests {

    use crate::Set;

    #[test]
    fn bitor_doc_test() {
        let a = Set::from([1, 2, 3]);
        let b = Set::from([3, 4, 5]);

        let set = &a | &b;

        let mut i = 0;
        let expected = [1, 2, 3, 4, 5];
        for x in &set {
            assert!(expected.contains(x));
            i += 1;
        }
        assert_eq!(i, expected.len());
    }

    #[test]
    fn bitand_doc_test() {
        let a = Set::from([1, 2, 3]);
        let b = Set::from([2, 3, 4]);

        let set = &a & &b;

        let mut i = 0;
        let expected = [2, 3];
        for x in &set {
            assert!(expected.contains(x));
            i += 1;
        }
        assert_eq!(i, expected.len());
    }

    #[test]
    fn bitxor_doc_test() {
        let a = Set::from([1, 2, 3]);
        let b = Set::from([3, 4, 5]);

        let set = &a ^ &b;

        let mut i = 0;
        let expected = [1, 2, 4, 5];
        for x in &set {
            assert!(expected.contains(x));
            i += 1;
        }
        assert_eq!(i, expected.len());
    }

    #[test]
    fn sub_doc_test() {
        let a = Set::from([1, 2, 3]);
        let b = Set::from([3, 4, 5]);

        let set = &a - &b;

        let mut i = 0;
        let expected = [1, 2];
        for x in &set {
            assert!(expected.contains(x));
            i += 1;
        }
        assert_eq!(i, expected.len());
    }
}
