

use crate::Set;
use crate::SetIter;

/// A lazy iterator producing elements in the difference of Liner `Set`s.
///
/// This `struct` is created by the [`difference`] method on [`Set`].
///
/// [`difference`]: Set::difference
///
/// # Examples
///
/// ```
/// use micromap::Set;
///
/// let a = Set::from([1, 2, 3]);
/// let b = Set::from([4, 2, 3, 4]);
///
/// let mut difference = a.difference(&b);
/// ```
pub struct Difference<'a, T: 'a + PartialEq, const N: usize> {
    // iterator of the first set
    pub(super) iter: SetIter<'a, T>,
    // the second set
    pub(super) other: &'a Set<T, N>,
}

impl<T: PartialEq, const N: usize> Clone for Difference<'_, T, N> {
    #[inline]
    fn clone(&self) -> Self {
        Difference {
            iter: self.iter.clone(),
            ..*self
        }
    }
}

impl<'a, T: PartialEq, const N: usize> Iterator for Difference<'a, T, N> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        for item in self.iter.by_ref() {
            if !self.other.contains(item) {
                return Some(item);
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
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

#[cfg(feature = "std")]
impl<T: std::fmt::Debug + PartialEq, const N: usize> std::fmt::Debug for Difference<'_, T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}


#[cfg(test)]
mod tests {
    use crate::Set;

    #[test]
    fn test_difference() {
        let set_a: Set<u32, 4> = Set::from([1, 3, 5, 7]);
        let set_b: Set<u32, 4> = Set::from([2, 4, 6, 8]);

        let set_diff = set_a.difference(&set_b).copied().collect::<Set<u32, 4>>();
        assert_eq!(set_a, set_diff);
    }

    #[test]
    fn test_difference_with_overlap() {
        let set_a: Set<u32, 4> = Set::from([1, 3, 5, 7]);
        let set_b: Set<u32, 4> = Set::from([3, 5, 6, 8]);

        let set_diff = set_a.difference(&set_b).copied().collect::<Set<u32, 4>>();
        let expected: Set<u32, 4> = Set::from_iter([1, 7]);
        assert_eq!(expected, set_diff);
    }

    #[test]
    fn test_difference_complete_overlap() {
        let set_a: Set<u32, 4> = Set::from([1, 3, 5, 7]);
        let set_b: Set<u32, 4> = Set::from([1, 3, 5, 7]);

        let set_diff = set_a.difference(&set_b).copied().collect::<Set<u32, 4>>();
        let expected: Set<u32, 4> = Set::from_iter([]);
        assert_eq!(expected, set_diff);
    }

    #[test]
    fn test_difference_empty_set() {
        let set_a: Set<u32, 4> = Set::from([1, 3, 5, 7]);
        let set_b: Set<u32, 4> = Set::from_iter([]);

        let set_diff = set_a.difference(&set_b).copied().collect::<Set<u32, 4>>();
        assert_eq!(set_a, set_diff);
    }

    #[test]
    fn test_difference_with_empty_first_set() {
        let set_a: Set<u32, 4> = Set::from_iter([]);
        let set_b: Set<u32, 4> = Set::from([2, 4, 6, 8]);

        let set_diff = set_a.difference(&set_b).copied().collect::<Set<u32, 4>>();
        let expected: Set<u32, 4> = Set::from_iter([]);
        assert_eq!(expected, set_diff);
    }

    #[test]
    fn test_difference_partial_overlap() {
        let set_a: Set<u32, 6> = Set::from([1, 2, 3, 4, 5, 6]);
        let set_b: Set<u32, 6> = Set::from([4, 5, 6, 7, 8, 9]);

        let set_diff = set_a.difference(&set_b).copied().collect::<Set<u32, 6>>();
        let expected: Set<u32, 6> = Set::from_iter([1, 2, 3]);
        assert_eq!(expected, set_diff);
    }
}