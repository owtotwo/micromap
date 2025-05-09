// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-FileCopyrightText: Copyright (c) 2025 owtotwo
// SPDX-License-Identifier: MIT

use super::iterators::Iter;
use super::Set;

impl<T: PartialEq, const N: usize> Set<T, N> {
    /// Visits the values representing the difference,
    /// i.e., the values that are in `self` but not in `other`.
    ///
    /// # Examples
    /// ```
    /// use micromap::Set;
    /// let a = Set::from([1, 2, 3]);
    /// let b = Set::from([4, 2, 3, 4]);
    /// // Can be seen as `a - b`.
    /// for x in a.difference(&b) {
    ///     println!("{x}"); // Print 1
    /// }
    /// let diff: Set<_, 3> = a.difference(&b).copied().collect();
    /// assert_eq!(diff, Set::from([1]));
    /// // Note that difference is not symmetric,
    /// // and `b - a` means something else:
    /// let diff: Set<_, 4> = b.difference(&a).copied().collect();
    /// assert_eq!(diff, Set::from([4]));
    /// ```
    #[inline]
    pub fn difference<'a, 'b, const M: usize>(
        &'a self,
        other: &'b Set<T, M>,
    ) -> Difference<'a, 'b, T, M> {
        Difference {
            iter: self.iter(),
            other,
        }
    }
}

/// A lazy iterator producing elements in the difference of Linear `Set`s.
///
/// This `struct` is created by the [`difference`] method on [`Set`].
///
/// [`difference`]: Set::difference
///
/// # Examples
/// ```
/// use micromap::Set;
/// let a = Set::from([1, 2, 3]);
/// let b = Set::from([4, 2, 3, 4]);
/// let diff = a.difference(&b);
/// assert_eq!(diff.count(), 1);
/// ```
#[must_use = "this returns the difference as an iterator, without modifying \
              either input set"]
pub struct Difference<'a, 'b, T, const M: usize> {
    // iterator of the first set
    iter: Iter<'a, T>,
    // the second set
    other: &'b Set<T, M>,
}

impl<T, const M: usize> Clone for Difference<'_, '_, T, M> {
    #[inline]
    fn clone(&self) -> Self {
        Difference {
            iter: self.iter.clone(),
            ..*self
        }
    }
}

impl<'a, T: PartialEq, const M: usize> Iterator for Difference<'a, '_, T, M> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find(|&item| !self.other.contains(item))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let lower = if self.iter.len() > self.other.len() {
            self.iter.len() - self.other.len()
        } else {
            0
        };
        let (_, upper) = self.iter.size_hint();
        (lower, upper)
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
                acc
            } else {
                f(acc, elt)
            }
        })
    }
}

impl<T: PartialEq, const M: usize> core::iter::FusedIterator for Difference<'_, '_, T, M> {}

impl<T: core::fmt::Debug + PartialEq, const M: usize> core::fmt::Debug
    for Difference<'_, '_, T, M>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

pub mod difference_ref {
    use super::{Iter, Set};

    impl<'a, T: PartialEq + ?Sized, const N: usize> Set<&'a T, N> {
        /// A [`difference()`][Set::difference] method with more elaborate lifetime and
        /// just for `Set<&T>`.
        #[inline]
        pub fn difference_ref<'b, 'set1, 'set2, const M: usize>(
            &'set1 self,
            other: &'set2 Set<&'b T, M>,
        ) -> DifferenceRef<'a, 'b, 'set2, T, M>
        where
            'set1: 'a,
            'set2: 'b,
        {
            DifferenceRef {
                iter: self.iter(),
                other,
            }
        }
    }

    /// A lazy iterator producing reference elements in the difference of
    /// Linear `Set`s.
    ///
    /// This `struct` is created by the [`difference_ref`] method on [`Set`].
    ///
    /// [`difference_ref`]: Set::difference_ref
    ///
    /// # Examples
    /// ```
    /// use micromap::Set;
    /// let a = Set::from(["foo", "bar", "baz"]);
    /// let b = Set::from(["fox", "foo", "baz", "bro"]);
    /// let diff = (&a).difference_ref(&b);
    /// assert_eq!(diff.count(), 1);
    /// ```
    #[must_use = "this returns the difference as an iterator, without modifying \
                  either input set"]
    pub struct DifferenceRef<'a, 'b, 'set, T: ?Sized, const M: usize> {
        // iterator of the first set
        iter: Iter<'a, &'a T>,
        // the second set
        other: &'set Set<&'b T, M>,
    }

    impl<T, const M: usize> Clone for DifferenceRef<'_, '_, '_, T, M> {
        #[inline]
        fn clone(&self) -> Self {
            DifferenceRef {
                iter: self.iter.clone(),
                ..*self
            }
        }
    }

    impl<'a, T: PartialEq + ?Sized, const M: usize> Iterator for DifferenceRef<'a, '_, '_, T, M> {
        type Item = &'a T;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.iter.find(|&item| !self.other.contains(item)).copied()
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let lower = if self.iter.len() > self.other.len() {
                self.iter.len() - self.other.len()
            } else {
                0
            };
            let (_, upper) = self.iter.size_hint();
            (lower, upper)
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
                    acc
                } else {
                    f(acc, elt)
                }
            })
        }
    }

    impl<T: PartialEq, const M: usize> core::iter::FusedIterator for DifferenceRef<'_, '_, '_, T, M> {}

    impl<T: core::fmt::Debug + PartialEq, const M: usize> core::fmt::Debug
        for DifferenceRef<'_, '_, '_, T, M>
    {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_list().entries(self.clone()).finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Set;

    // NOTE: This is a BUG in the standard library function.
    #[test]
    fn difference_lifetime() {
        // use std::collections::hash_set::HashSet as Set;
        let sentence_1 = String::from("I love the surf and the sand.");
        let sentence_1_words: Set<&str, 10> = sentence_1.split(" ").collect();
        let first_only = {
            let sentence_2 = String::from("I hate the hate and the sand.");
            let sentence_2_words: Set<&str, 10> = sentence_2.split(" ").collect();
            let first_only: Vec<_> = sentence_1_words.difference_ref(&sentence_2_words).collect();
            let second_only: Vec<_> = sentence_2_words.difference_ref(&sentence_1_words).collect();

            println!("First  Sentence: {}", sentence_1);
            println!("Second Sentence: {}", sentence_2);
            println!("{:?}", first_only);
            println!("{:?}", second_only);
            first_only
        };
        assert_eq!(
            Set::<_, 10>::from_iter(first_only.iter().copied()),
            Set::<_, 2>::from(["love", "surf"])
        );
    }

    #[test]
    fn difference_disjoint() {
        let set_a: Set<u32, 5> = Set::from([0, 1, 3, 5, 7]);
        let set_b: Set<u32, 4> = Set::from([2, 4, 6, 8]);
        let set_diff = set_a.difference(&set_b).copied().collect::<Set<u32, 5>>();
        assert_eq!(set_a, set_diff);
    }

    #[test]
    fn difference_with_overlap() {
        let set_a = Set::from([1, 3, 5, 7]);
        let set_b = Set::from([3, 5, 6, 8, 9]);
        let set_diff = set_a.difference(&set_b).copied().collect::<Set<u32, 4>>();
        let expected = Set::from([1, 7]);
        assert_eq!(expected, set_diff);
    }

    #[test]
    fn difference_complete_overlap() {
        let set_a = Set::from([1, 3, 5, 7]);
        let set_b = Set::from([1, 3, 5, 7]);
        let set_diff = set_a.difference(&set_b).copied().collect::<Set<u32, 4>>();
        let expected = Set::from([]);
        assert_eq!(expected, set_diff);
    }

    #[test]
    fn difference_empty_set() {
        let set_a = Set::from([1, 3, 5, 7]);
        let set_b = Set::from([]);
        let set_diff = set_a.difference(&set_b).copied().collect::<Set<u32, 4>>();
        assert_eq!(set_a, set_diff);
    }

    #[test]
    fn difference_with_empty_first_set() {
        let set_a = Set::from([]);
        let set_b = Set::from([2, 4, 6, 8]);
        let set_diff = set_a.difference(&set_b).copied().collect::<Set<u32, 4>>();
        let expected = Set::from([]);
        assert_eq!(expected, set_diff);
    }

    #[test]
    fn difference_partial_overlap() {
        let set_a = Set::from([1, 2, 3, 4, 5, 6]);
        let set_b = Set::from([4, 5, 6, 7, 8, 9]);
        let set_diff = set_a.difference(&set_b).copied().collect::<Set<_, 6>>();
        let expected = Set::from([1, 2, 3]);
        assert_eq!(expected, set_diff);
    }

    #[test]
    fn difference_size_hint() {
        let set_a = Set::from([1, 1, 2, 3]); // cap is 4, but len() is 3
        let set_b = Set::from([4, 5, 6, 6, 6, 7, 8, 9]); // cap is 8, but len() is 6
        let set_c = Set::from([]);
        let set_d = Set::from([3, 4]);
        assert_eq!(set_a.difference(&set_b).size_hint(), (0, Some(3)));
        assert_eq!(set_a.difference(&set_c).size_hint(), (3, Some(3)));
        assert_eq!(set_a.difference(&set_d).size_hint(), (1, Some(3)));
        assert_eq!(set_b.difference(&set_a).size_hint(), (3, Some(6)));
        assert_eq!(set_b.difference(&set_d).size_hint(), (4, Some(6)));
        assert_eq!(set_c.difference(&set_b).size_hint(), (0, Some(0)));
        assert_eq!(set_d.difference(&set_a).size_hint(), (0, Some(2)));
        assert_eq!(set_d.difference(&set_b).size_hint(), (0, Some(2)));
        assert_eq!(set_d.difference(&set_c).size_hint(), (2, Some(2)));
    }
}
