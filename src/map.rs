// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::{Drain, Entry, Map, OccupiedEntry, VacantEntry};
use core::borrow::Borrow;

impl<K, V, const N: usize> Map<K, V, N> {
    /// Get its total capacity.
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Is it empty?
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the total number of pairs inside.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Clears the map, returning all key-value pairs as an iterator. Keeps the allocated memory for reuse.
    ///
    /// If the returned iterator is dropped before being fully consumed, it drops the remaining key-value pairs. The returned iterator keeps a mutable borrow on the map to optimize its implementation.
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        let drain = Drain {
            iter: self.pairs[0..self.len].iter_mut(),
        };
        self.len = 0;
        drain
    }

    /// Remove all pairs from it, but keep the space intact for future use.
    #[inline]
    pub fn clear(&mut self) {
        for i in 0..self.len {
            unsafe { self.item_drop(i) };
        }
        self.len = 0;
    }

    /// Retains only the elements specified by the predicate.
    #[inline]
    pub fn retain<F: Fn(&K, &V) -> bool>(&mut self, f: F) {
        let mut i = 0;
        while i < self.len {
            let p = unsafe { self.item_ref(i) };
            if f(&p.0, &p.1) {
                // do not remove -> next index
                i += 1;
            } else {
                unsafe { self.remove_index_drop(i) };
                // recheck the same index
            }
        }
    }
}

impl<K: PartialEq, V, const N: usize> Map<K, V, N> {
    /// Does the map contain this key?
    #[inline]
    #[must_use]
    pub fn contains_key<Q: PartialEq + ?Sized>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        self.iter().any(|(x, _)| x.borrow() == k)
    }

    /// Remove by key.
    #[inline]
    pub fn remove<Q: PartialEq + ?Sized>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        for i in 0..self.len {
            let p = unsafe { self.item_ref(i) };
            if p.0.borrow() == k {
                return Some(unsafe { self.remove_index_read(i).1 });
            }
        }
        None
    }

    /// Insert a single pair into the map.
    ///
    /// # Panics
    ///
    /// It may panic if there are too many pairs in the map already.
    /// In order to comply with the memory safety of the Rust language itself, it will
    /// perform bounds checking, whether in `debug` mode or `release` mode.
    ///
    /// Because the implementation of this `insert()` mainly uses iterators instead of
    /// loops, it is not much slower in practice. It is even faster when frequently
    /// inserting and replacing pairs of existing keys which already in set.
    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let (_, existing_pair) = self.insert_ii(k, v);
        existing_pair.map(|(_, v)| v)
    }

    /// Attempt to insert a pair into the map. (no panic)
    ///
    /// - If the key existed, we update the pair, return `Some(old_value)`
    /// - If the key does not exist and the map has empty slot, we insert into that slot
    ///   and return `Some(None)`.
    /// - If the key does not exist and the map is full already, return `None`.
    #[inline]
    pub fn checked_insert(&mut self, k: K, v: V) -> Option<Option<V>> {
        if self.len < N {
            Some(self.insert_ii(k, v).1.map(|(_, v)| v))
        } else {
            self.insert_ii_for_full(k, v).map(|(_, (_, v))| Some(v))
        }
    }

    /// Insert a single pair into the map without bound check in release mode.
    ///
    /// # Panics
    ///
    /// It may panic if there are too many pairs in the map already. Pay attention,
    /// it panics only in the `debug` mode. In the `release` mode, you are going to get
    /// **undefined behavior**. This is done for the sake of performance, in order to
    /// avoid a repetitive check for the boundary condition on every `insert()`.
    ///
    /// # Safety
    ///
    /// Calling this method to add a new key-value pair when the [`Map`] is already
    /// full is undefined behavior instead of panic. So you need to make sure that
    /// the map is not full before calling.
    #[inline]
    pub unsafe fn insert_unchecked(&mut self, k: K, v: V) -> Option<V> {
        let (_, existing_pair) = self.insert_i(k, v);
        existing_pair.map(|(_, v)| v)
    }

    /// Get a reference to a single value.
    #[inline]
    #[must_use]
    pub fn get<Q: PartialEq + ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        for i in 0..self.len {
            let p = unsafe { self.item_ref(i) };
            if p.0.borrow() == k {
                return Some(&p.1);
            }
        }
        None
    }

    /// Get a mutable reference to a single value.
    ///
    /// # Panics
    ///
    /// If can't turn it into a mutable state.
    #[inline]
    #[must_use]
    pub fn get_mut<Q: PartialEq + ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        for i in 0..self.len {
            let p = unsafe { self.item_ref(i) };
            if p.0.borrow() == k {
                return Some(unsafe { self.item_mut(i) });
            }
        }
        None
    }

    /// Returns the key-value pair corresponding to the supplied key.
    #[inline]
    pub fn get_key_value<Q: PartialEq + ?Sized>(&self, k: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
    {
        for i in 0..self.len {
            let p = unsafe { self.item_ref(i) };
            if p.0.borrow() == k {
                return Some((&p.0, &p.1));
            }
        }
        None
    }

    /// Attempts to get mutable references to `J` values in the map at once.
    /// Or you want to batch search keys (and get mutable references to them) in
    /// one iteration for map.
    ///
    /// Returns an array of length `J` with the results of each query.
    /// For soundness, at most one mutable reference will be returned to any value.
    /// None will be used if the key is missing.
    ///
    /// # Panics
    ///
    /// Panics if any keys are overlapping.
    ///
    /// # Examples
    ///
    /// ```
    /// use micromap::Map;
    ///
    /// let mut libraries: Map<String, u32, 5> = Map::new();
    /// libraries.insert("Bodleian Library".to_string(), 1602);
    /// libraries.insert("Athenæum".to_string(), 1807);
    /// libraries.insert("Herzogin-Anna-Amalia-Bibliothek".to_string(), 1691);
    /// libraries.insert("Library of Congress".to_string(), 1800);
    ///
    /// // Get Athenæum and Bodleian Library
    /// let [Some(a), Some(b)] = libraries.get_disjoint_mut([
    ///     "Athenæum",
    ///     "Bodleian Library",
    /// ]) else { panic!() };
    ///
    /// // Assert values of Athenæum and Library of Congress
    /// let got = libraries.get_disjoint_mut([
    ///     "Athenæum",
    ///     "Library of Congress",
    /// ]);
    /// assert_eq!(
    ///     got,
    ///     [
    ///         Some(&mut 1807),
    ///         Some(&mut 1800),
    ///     ],
    /// );
    ///
    /// // Missing keys result in None
    /// let got = libraries.get_disjoint_mut([
    ///     "Athenæum",
    ///     "New York Public Library",
    /// ]);
    /// assert_eq!(
    ///     got,
    ///     [
    ///         Some(&mut 1807),
    ///         None
    ///     ]
    /// );
    /// ```
    ///
    /// ```should_panic
    /// use std::collections::HashMap;
    ///
    /// let mut libraries = HashMap::new();
    /// libraries.insert("Athenæum".to_string(), 1807);
    ///
    /// // Duplicate keys panic!
    /// let got = libraries.get_disjoint_mut([
    ///     "Athenæum",
    ///     "Athenæum",
    /// ]);
    /// ```
    #[inline]
    pub fn get_disjoint_mut<Q, const J: usize>(&mut self, ks: [&Q; J]) -> [Option<&mut V>; J]
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        if ks.len() == 0 {
            return [const { None }; J];
        }

        // check for overlapping keys (O(n^2), but n is small)
        for (i, k) in ks[..ks.len() - 1].iter().enumerate() {
            for k_behind in ks[i + 1..].iter() {
                assert!(k != k_behind, "Overlapping keys");
            }
        }

        unsafe { self.get_disjoint_unchecked_mut(ks) }
    }

    /// Attempts to get mutable references to `J` values in the map at once, without validating that
    /// the values are unique. Or you want to batch search keys (and get mutable references to them)
    /// in one iteration for map.
    ///
    /// Returns an array of length `J` with the results of each query. `None` will be used if
    /// the key is missing.
    ///
    /// For a safe alternative see [`Map::get_disjoint_mut`].
    ///
    /// # Safety
    ///
    /// Calling this method with overlapping keys is *[undefined behavior]* even if the resulting
    /// references are not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    ///
    /// # Examples
    ///
    /// ```
    /// use micromap::Map;
    ///
    /// let mut libraries: Map<String, u32, 5> = Map::new();
    /// libraries.insert("Bodleian Library".to_string(), 1602);
    /// libraries.insert("Athenæum".to_string(), 1807);
    /// libraries.insert("Herzogin-Anna-Amalia-Bibliothek".to_string(), 1691);
    /// libraries.insert("Library of Congress".to_string(), 1800);
    ///
    /// // SAFETY: The keys do not overlap.
    /// let [Some(a), Some(b)] = (unsafe { libraries.get_disjoint_unchecked_mut([
    ///     "Athenæum",
    ///     "Bodleian Library",
    /// ]) }) else { panic!() };
    ///
    /// // SAFETY: The keys do not overlap.
    /// let got = unsafe { libraries.get_disjoint_unchecked_mut([
    ///     "Athenæum",
    ///     "Library of Congress",
    /// ]) };
    /// assert_eq!(
    ///     got,
    ///     [
    ///         Some(&mut 1807),
    ///         Some(&mut 1800),
    ///     ],
    /// );
    ///
    /// // SAFETY: The keys do not overlap.
    /// let got = unsafe { libraries.get_disjoint_unchecked_mut([
    ///     "Athenæum",
    ///     "New York Public Library",
    /// ]) };
    /// // Missing keys result in None
    /// assert_eq!(got, [Some(&mut 1807), None]);
    /// ```
    #[inline]
    pub unsafe fn get_disjoint_unchecked_mut<Q, const J: usize>(
        &mut self,
        ks: [&Q; J],
    ) -> [Option<&mut V>; J]
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        let mut ret: [Option<&mut V>; J] = [const { None }; J];

        if ks.len() == 0 {
            return ret;
        } else if ks.len() == 1 {
            ret[0] = self.get_mut(ks[0]);
            return ret;
        }

        // find the keys' index in one iteration of the map, store the result
        // into the stack.
        let mut stack = [const { None }; J];
        let mut stack_top = 0;
        for pair_i in 0..self.len {
            let p = unsafe { self.item_ref(pair_i) };
            if let Some(ks_i) = ks.iter().position(|&k| k.borrow() == p.0.borrow()) {
                stack[stack_top] = Some((pair_i, ks_i));
                stack_top += 1;
                continue;
            }
        }

        stack[..stack_top].sort_unstable_by_key(|x| x.map(|(pair_i, _)| pair_i));

        // start splitting the found pairs from the back to the front
        let mut rest_head = &mut self.pairs[..self.len];
        for found in stack[..stack_top].iter().rev() {
            if let Some((pair_i, ks_i)) = found {
                let (head, tail) = rest_head.split_at_mut(*pair_i);
                rest_head = head;
                let p = unsafe { tail[0].assume_init_mut() };
                ret[*ks_i] = Some(&mut p.1);
            }
        }
        ret
    }

    /// Removes a key from the map, returning the stored key and value if the
    /// key was previously in the map.
    #[inline]
    pub fn remove_entry<Q: PartialEq + ?Sized>(&mut self, k: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
    {
        for i in 0..self.len {
            let p = unsafe { self.item_ref(i) };
            if p.0.borrow() == k {
                return Some(unsafe { self.remove_index_read(i) });
            }
        }
        None
    }

    pub fn entry(&mut self, k: K) -> Entry<'_, K, V, N> {
        for i in 0..self.len {
            let p = unsafe { self.item_ref(i) };
            if p.0 == k {
                return Entry::Occupied(OccupiedEntry {
                    index: i,
                    table: self,
                });
            }
        }
        Entry::Vacant(VacantEntry {
            key: k,
            table: self,
        })
    }
}

mod internal {
    use crate::Map;

    impl<K, V, const N: usize> Map<K, V, N> {
        /// Internal function to get access via reference to the element in the internal array.
        #[inline]
        pub(crate) unsafe fn item_ref(&self, i: usize) -> &(K, V) {
            self.pairs.get_unchecked(i).assume_init_ref()
        }

        /// Internal function to get mutable access via reference to the element in the internal array.
        #[inline]
        pub(crate) unsafe fn item_mut(&mut self, i: usize) -> &mut V {
            &mut self.pairs.get_unchecked_mut(i).assume_init_mut().1
        }

        /// Internal function to get access to the element in the internal array.
        #[inline]
        pub(crate) unsafe fn item_read(&mut self, i: usize) -> (K, V) {
            self.pairs.get_unchecked(i).assume_init_read()
        }

        /// Internal function to get access to the element in the internal array.
        #[inline]
        pub(crate) unsafe fn item_drop(&mut self, i: usize) {
            self.pairs.get_unchecked_mut(i).assume_init_drop();
        }

        /// Internal function to get access to the element in the internal array.
        #[inline]
        pub(crate) unsafe fn item_write(&mut self, i: usize, val: (K, V)) {
            self.pairs.get_unchecked_mut(i).write(val);
        }

        /// Remove an index (by swapping the last one here and reducing the length)
        #[inline]
        pub(crate) unsafe fn remove_index_drop(&mut self, i: usize) {
            self.item_drop(i);

            self.len -= 1;
            if i != self.len {
                let value = self.item_read(self.len);
                self.item_write(i, value);
            }
        }

        /// Remove an index (by swapping the last one here and reducing the length)
        #[inline]
        pub(crate) unsafe fn remove_index_read(&mut self, i: usize) -> (K, V) {
            let result = self.item_read(i);

            self.len -= 1;
            if i != self.len {
                let value = self.item_read(self.len);
                self.item_write(i, value);
            }

            result
        }
    }

    impl<K: PartialEq, V, const N: usize> Map<K, V, N> {
        /// The core insert logic, which is used for `insert_unchecked()`, as it will
        /// disable the bound check (`debug_assert!`) in `release` mode.
        pub(crate) fn insert_i(&mut self, k: K, v: V) -> (usize, Option<(K, V)>) {
            let mut target = self.len;
            let mut i = 0;
            let mut existing_pair = None;
            loop {
                if i == self.len {
                    core::debug_assert!(target < N, "No more key-value slot available in the map");
                    break;
                }
                let p = unsafe { self.item_ref(i) };
                if p.0 == k {
                    target = i;
                    existing_pair = Some(unsafe { self.item_read(i) });
                    break;
                }
                i += 1;
            }
            unsafe { self.item_write(target, (k, v)) };
            if target == self.len {
                self.len += 1;
            }

            (target, existing_pair)
        }

        /// The core insert logic, which is used for `insert()`. Its name means
        /// that it uses iterators(the second `i`) instead of loops to implement the underlying
        /// insertion logic for `insert_i()`.
        pub(crate) fn insert_ii(&mut self, k: K, v: V) -> (usize, Option<(K, V)>) {
            if let Some((i, pair)) = self.pairs[..self.len]
                .iter_mut()
                .map(|p| unsafe { p.assume_init_mut() })
                .enumerate()
                .find(|(_i, p)| p.0 == k)
            {
                (i, Some(core::mem::replace(pair, (k, v))))
            } else {
                let i = self.len;
                // just for panic msg in debug mode, not the main bound check
                core::debug_assert!(i < N, "No more key-value slot available in the map");
                self.pairs[i].write((k, v)); // main bound check here but it's ok (not hotspot)
                self.len += 1;
                (i, None)
            }
        }

        /// When the map is full, we need to check the key-value pair if existed already or not.
        /// If no place to insert the new key-value pair, we return `None`.
        pub(crate) fn insert_ii_for_full(&mut self, k: K, v: V) -> Option<(usize, (K, V))> {
            if let Some((i, pair)) = self.pairs[..self.len]
                .iter_mut()
                .map(|p| unsafe { p.assume_init_mut() })
                .enumerate()
                .find(|(_, p)| p.0 == k)
            {
                let existing_pair = core::mem::replace(pair, (k, v));
                Some((i, existing_pair))
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn insert_and_check_length() {
        let mut m: Map<String, i32, 10> = Map::new();
        assert_eq!(m.insert("first".to_string(), 42), None);
        assert_eq!(1, m.len());
        assert_eq!(m.insert("second".to_string(), 16), None);
        assert_eq!(2, m.len());
        assert_eq!(m.insert("first".to_string(), 16), Some(42));
        assert_eq!(2, m.len());
    }

    #[test]
    fn overwrites_keys() {
        let mut m: Map<i32, i32, 1> = Map::new();
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.insert(1, 42), Some(42));
        assert_eq!(1, m.len());
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn cant_write_into_empty_map() {
        let mut m: Map<i32, i32, 0> = Map::new();
        assert_eq!(m.insert(1, 42), None);
    }

    #[test]
    fn empty_length() {
        let m: Map<u32, u32, 10> = Map::new();
        assert_eq!(0, m.len());
    }

    #[test]
    fn is_empty_check() {
        let mut m: Map<u32, u32, 10> = Map::new();
        assert!(m.is_empty());
        assert_eq!(m.insert(42, 42), None);
        assert!(!m.is_empty());
    }

    #[test]
    fn insert_and_gets() {
        let mut m: Map<String, i32, 10> = Map::new();
        assert_eq!(m.insert("one".to_string(), 42), None);
        assert_eq!(m.insert("two".to_string(), 16), None);
        assert_eq!(16, *m.get("two").unwrap());
    }

    #[test]
    fn insert_and_gets_mut() {
        let mut m: Map<i32, [i32; 3], 10> = Map::new();
        assert_eq!(m.insert(42, [1, 2, 3]), None);
        let a = m.get_mut(&42).unwrap();
        a[0] = 500;
        assert_eq!(500, m.get(&42).unwrap()[0]);
    }

    #[test]
    fn checks_key() {
        let mut m: Map<String, i32, 10> = Map::new();
        assert_eq!(m.insert("one".to_string(), 42), None);
        assert!(m.contains_key("one"));
        assert!(!m.contains_key("another"));
    }

    #[test]
    fn gets_missing_key() {
        let mut m: Map<String, i32, 10> = Map::new();
        assert_eq!(m.insert("one".to_string(), 42), None);
        assert!(m.get("two").is_none());
    }

    #[test]
    fn mut_gets_missing_key() {
        let mut m: Map<String, i32, 10> = Map::new();
        assert_eq!(m.insert("one".to_string(), 42), None);
        assert!(m.get_mut("two").is_none());
    }

    #[test]
    fn removes_simple_pair() {
        let mut m: Map<String, i32, 10> = Map::new();
        assert_eq!(m.insert("one".to_string(), 42), None);
        assert_eq!(m.remove("one"), Some(42));
        assert_eq!(m.remove("another"), None);
        assert!(m.get("one").is_none());
    }

    #[cfg(test)]
    #[derive(Clone, PartialEq, Debug)]
    struct Foo {
        v: [u32; 3],
    }

    #[test]
    fn insert_struct() {
        let mut m: Map<u8, Foo, 8> = Map::new();
        let foo = Foo { v: [1, 2, 100] };
        assert_eq!(m.insert(1, foo), None);
        assert_eq!(100, m.into_iter().next().unwrap().1.v[2]);
    }

    #[cfg(test)]
    #[derive(Clone, PartialEq, Debug)]
    struct Composite {
        r: Map<u8, u8, 1>,
    }

    #[test]
    fn insert_composite() {
        let mut m: Map<u8, Composite, 8> = Map::new();
        let c = Composite { r: Map::new() };
        assert_eq!(m.insert(1, c), None);
        assert_eq!(0, m.into_iter().next().unwrap().1.r.len());
    }

    #[test]
    fn large_map_in_heap() {
        let m: Box<Map<u64, [u64; 10], 10>> = Box::new(Map::new());
        assert_eq!(0, m.len());
    }

    #[test]
    fn clears_it_up() {
        let mut m: Map<String, i32, 10> = Map::new();
        assert_eq!(m.insert("one".to_string(), 42), None);
        m.clear();
        assert_eq!(0, m.len());
    }

    #[test]
    fn retain_test() {
        let vec: Vec<(i32, i32)> = (0..8).map(|x| (x, x * 10)).collect();
        let mut m: Map<i32, i32, 10> = Map::from_iter(vec);
        assert_eq!(m.len(), 8);
        m.retain(|&k, _| k < 6);
        assert_eq!(m.len(), 6);
        m.retain(|_, &v| v > 30);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn insert_many_and_remove() {
        let mut m: Map<usize, u64, 4> = Map::new();
        for _ in 0..2 {
            let cap = m.capacity();
            for i in 0..cap {
                assert_eq!(m.insert(i, 256), None);
                assert_eq!(m.remove(&i), Some(256));
            }
        }
    }

    #[test]
    fn get_key_value() {
        let mut m: Map<String, i32, 10> = Map::new();
        let k = "key".to_string();
        assert_eq!(m.insert(k.clone(), 42), None);
        assert_eq!(m.get_key_value(&k), Some((&k, &42)));
        assert!(m.contains_key(&k));
    }

    #[test]
    fn get_absent_key_value() {
        let mut m: Map<String, i32, 10> = Map::new();
        assert_eq!(m.insert("one".to_string(), 42), None);
        assert_eq!(m.get_key_value("two"), None);
    }

    #[test]
    fn remove_entry_present() {
        let mut m: Map<String, i32, 10> = Map::new();
        let k = "key".to_string();
        assert_eq!(m.insert(k.clone(), 42), None);
        assert_eq!(m.remove_entry(&k), Some((k.clone(), 42)));
        assert!(!m.contains_key(&k));
    }

    #[test]
    fn remove_entry_absent() {
        let mut m: Map<String, i32, 10> = Map::new();
        assert_eq!(m.insert("one".to_string(), 42), None);
        assert_eq!(m.remove_entry("two"), None);
    }

    #[test]
    fn drop_removed_entry() {
        use std::rc::Rc;
        let mut m: Map<(), Rc<()>, 8> = Map::new();
        let v = Rc::new(());
        assert_eq!(m.insert((), Rc::clone(&v)), None);
        assert_eq!(Rc::strong_count(&v), 2);
        assert_eq!(m.remove_entry(&()), Some(((), Rc::clone(&v))));
        assert_eq!(Rc::strong_count(&v), 1);
    }

    #[test]
    fn insert_after_remove() {
        let mut m: Map<_, _, 1> = Map::new();
        assert_eq!(m.insert(1, 2), None);
        assert_eq!(m.remove(&1), Some(2));
        assert_eq!(m.insert(1, 3), None);
    }

    #[test]
    fn insert_drop_duplicate() {
        use std::rc::Rc;
        let mut m: Map<_, _, 1> = Map::new();
        let v = Rc::new(());
        assert_eq!(m.insert((), Rc::clone(&v)), None);
        assert_eq!(Rc::strong_count(&v), 2);
        assert_eq!(m.insert((), Rc::clone(&v)), Some(Rc::clone(&v)));
        assert_eq!(Rc::strong_count(&v), 2);
    }

    #[test]
    fn insert_duplicate_after_remove() {
        let mut m: Map<_, _, 2> = Map::new();
        assert_eq!(m.insert(1, 1), None);
        assert_eq!(m.insert(2, 2), None);
        assert_eq!(m.remove(&1), Some(1));
        assert_eq!(m.insert(2, 3), Some(2));
        assert_eq!(1, m.len());
        assert_eq!(3, m[&2]);
    }

    #[test]
    fn checked_insert_updates_existing_key() {
        let mut m: Map<String, i32, 10> = Map::new();
        assert_eq!(m.checked_insert("key".to_string(), 42), Some(None));
        assert_eq!(m.checked_insert("key".to_string(), 100), Some(Some(42)));
        assert_eq!(m.get("key"), Some(&100));
    }

    #[test]
    fn checked_insert_inserts_new_key_when_space_available() {
        let mut m: Map<String, i32, 2> = Map::new();
        assert_eq!(m.checked_insert("key1".to_string(), 42), Some(None));
        assert_eq!(m.checked_insert("key2".to_string(), 100), Some(None));
        assert_eq!(m.get("key1"), Some(&42));
        assert_eq!(m.get("key2"), Some(&100));
    }

    #[test]
    fn checked_insert_fails_when_map_is_full() {
        let mut m: Map<String, i32, 1> = Map::new();
        assert_eq!(m.checked_insert("key1".to_string(), 42), Some(None));
        assert_eq!(m.checked_insert("key2".to_string(), 100), None);
        assert_eq!(m.get("key1"), Some(&42));
        assert_eq!(m.get("key2"), None);
        assert_eq!(m.checked_insert("key1".to_string(), 43), Some(Some(42)));
        assert_eq!(m.get("key1"), Some(&43));
    }

    #[test]
    fn checked_insert_handles_empty_map() {
        let mut m: Map<String, i32, 0> = Map::new();
        assert_eq!(m.checked_insert("key".to_string(), 42), None);
        assert!(m.is_empty());
    }

    #[test]
    fn checked_insert_replaces_value_for_existing_key() {
        let mut m: Map<i32, i32, 3> = Map::new();
        assert_eq!(m.checked_insert(1, 10), Some(None));
        assert_eq!(m.checked_insert(1, 20), Some(Some(10)));
        assert_eq!(m.get(&1), Some(&20));
    }

    #[test]
    fn checked_insert_does_not_affect_other_keys() {
        let mut m: Map<i32, i32, 3> = Map::new();
        assert_eq!(m.checked_insert(1, 10), Some(None));
        assert_eq!(m.checked_insert(2, 20), Some(None));
        assert_eq!(m.checked_insert(1, 30), Some(Some(10)));
        assert_eq!(m.get(&1), Some(&30));
        assert_eq!(m.get(&2), Some(&20));
    }

    #[test]
    fn get_disjoint_mut_non_overlapping_keys() {
        let mut map: Map<&str, i32, 5> = Map::new();
        map.insert("key1", 10);
        map.insert("key2", 20);
        map.insert("key3", 30);
        map.insert("key4", 40);

        let [v3, v1, v2] = map.get_disjoint_mut(["key3", "key1", "key2"]);
        assert_eq!(v1, Some(&mut 10));
        assert_eq!(v2, Some(&mut 20));
        assert_eq!(v3, Some(&mut 30));

        if let Some(v1) = v1 {
            *v1 = 100;
        }
        if let Some(v2) = v2 {
            *v2 = 200;
        }
        if let Some(v3) = v3 {
            *v3 = 300;
        }

        assert_eq!(map.get("key1"), Some(&100));
        assert_eq!(map.get("key2"), Some(&200));
        assert_eq!(map.get("key3"), Some(&300));
        assert_eq!(map.get("key4"), Some(&40));
    }

    #[test]
    #[should_panic(expected = "Overlapping keys")]
    fn get_disjoint_mut_overlapping_keys() {
        let mut map: Map<&str, i32, 5> = Map::new();
        map.insert("key1", 10);
        map.insert("key2", 20);

        // This should panic because the keys overlap.
        let _ = map.get_disjoint_mut(["key1", "key1"]);
    }

    #[test]
    fn get_disjoint_mut_missing_keys() {
        let mut map: Map<&str, i32, 5> = Map::new();
        map.insert("key2", 20);

        let [v1, v2] = map.get_disjoint_mut(["key1", "key2"]);
        assert_eq!(v1, None);
        assert_eq!(v2, Some(&mut 20));
    }

    #[test]
    fn get_disjoint_mut_empty_keys() {
        let mut map: Map<&str, i32, 5> = Map::new();
        let result: [Option<&mut i32>; 0] = map.get_disjoint_mut::<&str, 0>([]);
        assert!(result.is_empty());
    }

    #[test]
    fn get_disjoint_mut_keys_more_than_capacity() {
        let mut map: Map<&str, i32, 3> = Map::new();
        assert!(map.is_empty());
        let [v1] = map.get_disjoint_mut(["key1"]);
        assert_eq!(v1, None);
        map.insert("key1", 10);
        let [v1] = map.get_disjoint_mut(["key1"]);
        assert_eq!(v1, Some(&mut 10));
        map.insert("key2", 20);
        map.insert("key3", 30);
        assert!(!map.is_empty());
        let result = map.get_disjoint_mut(["key1", "key2", "key3"]);
        result.iter().all(|x| x.is_some());
        let [v0, v1, v2, v3, v4] = map.get_disjoint_mut(["key0", "key1", "key2", "key3", "key4"]);
        assert_eq!(v0, None);
        assert_eq!(v1, Some(&mut 10));
        assert_eq!(v2, Some(&mut 20));
        assert_eq!(v3, Some(&mut 30));
        assert_eq!(v4, None);
    }
}
