// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-FileCopyrightText: Copyright (c) 2025 owtotwo
// SPDX-License-Identifier: MIT

use super::iterators::Iter;
use super::Set;

impl<T: PartialEq, const N: usize> Set<T, N> {
    /// Visits the values representing the intersection,
    /// i.e., the values that are both in `self` and `other`.
    ///
    /// When an equal element is present in `self` and `other`,
    /// unlike the standard library functions, the resulting `Intersection`
    /// will ALWAYS yield references to the caller(`self`). This can be
    /// relevant if `T` contains fields which are not compared by its `Eq`
    /// implementation, and may hold different value between the two equal
    /// copies of `T` in the two sets.
    ///
    /// # Examples
    /// ```
    /// use micromap::Set;
    /// let a = Set::from([1, 2, 3]);
    /// let b = Set::from([4, 2, 3, 4]);
    /// // Print 2, 3 in arbitrary order.
    /// for x in a.intersection(&b) {
    ///     println!("{x}");
    /// }
    /// let intersection: Set<_, 3> = a.intersection(&b).copied().collect();
    /// assert_eq!(intersection, Set::from([2, 3]));
    /// ```
    #[inline]
    pub fn intersection<'a, const M: usize>(
        &'a self,
        other: &'a Set<T, M>,
    ) -> Intersection<'a, T, M> {
        Intersection {
            iter: self.iter(),
            other,
        }
    }
}

/// A lazy iterator producing elements in the intersection of Linear `Set`s.
///
/// This `struct` is created by the [`intersection`] method on [`Set`].
/// See its documentation for more.
///
/// [`intersection`]: Set::intersection
///
/// # Examples
/// ```
/// use micromap::Set;
/// let a = Set::from([1, 2, 3]);
/// let b = Set::from([4, 2, 3, 4]);
/// let intersection = a.intersection(&b);
/// assert_eq!(intersection.count(), 2);
/// ```
#[must_use = "this returns the intersection as an iterator, without modifying \
              either input set"]
pub struct Intersection<'a, T, const M: usize> {
    // iterator of the first set
    iter: Iter<'a, T>,
    // the second set
    other: &'a Set<T, M>,
}

impl<T, const M: usize> Clone for Intersection<'_, T, M> {
    #[inline]
    fn clone(&self) -> Self {
        Intersection {
            iter: self.iter.clone(),
            ..*self
        }
    }
}

impl<'a, T: PartialEq, const M: usize> Iterator for Intersection<'a, T, M> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.by_ref().find(|&item| self.other.contains(item))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(usize::min(self.iter.len(), self.other.len())))
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        // Maybe using iterator is better than the default Iterator::fold() which uses while loop.
        self.iter.fold(init, |acc, elt| {
            if self.other.contains(elt) {
                f(acc, elt)
            } else {
                acc
            }
        })
    }
}

impl<T: PartialEq, const M: usize> core::iter::FusedIterator for Intersection<'_, T, M> {}

impl<T: PartialEq + core::fmt::Debug, const M: usize> core::fmt::Debug for Intersection<'_, T, M> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::Set;

    #[test]
    fn intersection_simple() {
        let set_a = Set::from([0, 1, 2, 3, 5, 7, 9]);
        let set_b = Set::from([2, 5, 6, 7, 8, 10]);
        let set_c = Set::from([0, 1, 2, 3, 5, 7, 9, 2, 5, 6, 7, 8]);
        let set_result: Set<_, 5> = set_a.intersection(&set_b).copied().collect();
        assert_eq!(set_result, Set::from([2, 5, 7]));
        let set_result: Set<_, 7> = set_a.intersection(&set_c).copied().collect();
        assert_eq!(set_result, set_a);
        let set_result: Set<_, 6> = set_b.intersection(&set_c).copied().collect();
        assert_eq!(set_result, Set::from([2, 5, 6, 7, 8]));
    }

    #[test]
    fn intersection_with_empty_set() {
        let a = Set::from([1, 2, 3]);
        let b: Set<i32, 3> = Set::new();
        let intersection: Set<_, 3> = a.intersection(&b).collect();
        assert!(intersection.is_empty());
    }

    #[test]
    fn intersection_with_disjoint_sets() {
        let a = Set::from([1, 2, 3]);
        let b = Set::from([4, 5, 6]);
        let intersection: Set<_, 3> = a.intersection(&b).copied().collect();
        assert!(intersection.is_empty());
    }

    #[test]
    fn intersection_with_subset() {
        let a = Set::from([1, 2, 3, 4]);
        let b = Set::from([2, 3]);
        let intersection: Set<_, 2> = a.intersection(&b).copied().collect();
        assert_eq!(intersection, Set::from([2, 3]));
    }

    #[test]
    fn intersection_with_superset() {
        let a = Set::from([2, 3]);
        let b = Set::from([1, 2, 3, 4]);
        let intersection: Set<_, 2> = a.intersection(&b).copied().collect();
        assert_eq!(intersection, Set::from([2, 3]));
    }

    #[test]
    fn intersection_size_hint() {
        let set_a = Set::from([1, 1, 2, 3]); // cap is 4, but len() is 3
        let set_b = Set::from([4, 5, 6, 6, 6, 7, 8, 9]); // cap is 8, but len() is 6
        let set_c = Set::from([]);
        let set_d = Set::from([3, 4]);
        assert_eq!(set_a.intersection(&set_b).size_hint(), (0, Some(3)));
        assert_eq!(set_a.intersection(&set_c).size_hint(), (0, Some(0)));
        assert_eq!(set_a.intersection(&set_d).size_hint(), (0, Some(2)));
        assert_eq!(set_b.intersection(&set_a).size_hint(), (0, Some(3)));
        assert_eq!(set_b.intersection(&set_d).size_hint(), (0, Some(2)));
        assert_eq!(set_c.intersection(&set_b).size_hint(), (0, Some(0)));
        assert_eq!(set_d.intersection(&set_a).size_hint(), (0, Some(2)));
        assert_eq!(set_d.intersection(&set_b).size_hint(), (0, Some(2)));
        assert_eq!(set_d.intersection(&set_c).size_hint(), (0, Some(0)));
    }

    #[test]
    fn intersection_clone_fmt_fold() {
        use core::fmt::Write;
        let set_a = Set::from([1, 2, 3, 4]);
        let set_b = Set::from([3, 4, 5, 6]);
        // Test `clone`
        let intersection = set_a.intersection(&set_b);
        let cloned = intersection.clone();
        let result: Set<_, 2> = cloned.copied().collect();
        assert_eq!(result, Set::from([3, 4]));
        // Test `fmt`
        let mut debug_output = String::new();
        write!(&mut debug_output, "{:?}", intersection).unwrap();
        assert!(debug_output.contains("3"));
        assert!(debug_output.contains("4"));
        // Test `fold`
        let sum = intersection.fold(0, |acc, &x| acc + x);
        assert_eq!(sum, 7); // 3 + 4 = 7
    }
}
