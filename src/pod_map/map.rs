// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT


use bytemuck::Pod;

use super::{Drain, Entry, PodMap, OccupiedEntry, VacantEntry};
use core::{borrow::Borrow, mem::replace};

mod internal {
    use core::mem::zeroed;

    use bytemuck::Pod;

    use super::PodMap;

    #[inline]
    pub(crate) const fn bit_size_of<K: Sized>() -> usize {
        core::mem::size_of::<K>() * 8
    }

    #[inline]
    pub(crate) fn count_ones<K: Pod>(k: &K) -> usize {
        let key = bytemuck::bytes_of(k);
        key.iter().map(|k| k.count_ones()).sum::<u32>() as usize
    }

    #[inline]
    pub(crate) fn more_zeros<K: Pod>(k: &K) -> bool {
        let total_len = bit_size_of::<K>();
        let ones_len = count_ones(k);
        let zero_len = total_len - ones_len;
        zero_len > ones_len
    }

    impl<K: PartialEq + Pod, V, const N: usize> PodMap<K, V, N> {
        #[inline]
        pub(crate) fn mark_check(&self, k: &K) -> bool {
            let key = bytemuck::bytes_of(k);
            if more_zeros(k) {
                let zeros = bytemuck::bytes_of(&self.bits.0);
                zeros
                    .iter()
                    .zip(key.iter())
                    .all(|(&zero, &k)| zero & k == k)
            } else {
                let ones = bytemuck::bytes_of(&self.bits.1);
                ones.iter().zip(key.iter()).all(|(&one, &k)| one & !k == !k)
            }
        }

        #[inline]
        pub(crate) fn mark_add(&mut self, k: usize) {
            let key = bytemuck::bytes_of(&k);
            if more_zeros(&k) {
                let zeros = bytemuck::bytes_of_mut(&mut self.bits.0);
                zeros
                    .iter_mut()
                    .zip(key.iter())
                    .for_each(|(zero, &k)| *zero |= k);
            } else {
                let ones = bytemuck::bytes_of_mut(&mut self.bits.1);
                ones.iter_mut()
                    .zip(key.iter())
                    .for_each(|(one, &k)| *one |= !k);
            }
        }

        #[inline]
        pub(crate) fn remark(&mut self) {
            self.bits = unsafe { (zeroed(), zeroed()) };
            let zeros = bytemuck::bytes_of_mut(&mut self.bits.0);
            let ones = bytemuck::bytes_of_mut(&mut self.bits.1);

            self.pairs[..self.len]
                .iter()
                .map(|p| unsafe { p.assume_init_ref().0 })
                .for_each(|k| {
                    if more_zeros(&k) {
                        zeros
                            .iter_mut()
                            .zip(bytemuck::bytes_of(&k).iter())
                            .for_each(|(zero, &k)| *zero |= k);
                    } else {
                        ones.iter_mut()
                            .zip(bytemuck::bytes_of(&k).iter())
                            .for_each(|(one, &k)| *one |= !k);
                    }
                });
        }
    }

    impl<K: PartialEq + Pod, V, const N: usize> PodMap<K, V, N> {
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
}

impl<K: PartialEq + Pod, V, const N: usize> PodMap<K, V, N> {
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

    /// Clears the PodMap, returning all key-value pairs as an iterator. Keeps the allocated memory for reuse.
    ///
    /// If the returned iterator is dropped before being fully consumed, it drops the remaining key-value pairs. The returned iterator keeps a mutable borrow on the PodMap to optimize its implementation.
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        let drain = Drain {
            iter: self.pairs[0..self.len].iter_mut(),
        };
        self.len = 0;
        drain
    }

    /// Does the PodMap contain this key?
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

    /// Insert a single pair into the PodMap.
    ///
    /// # Panics
    ///
    /// It may panic if there are too many pairs in the PodMap already.
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

    /// Insert a single pair into the PodMap without bound check in release mode.
    ///
    /// # Panics
    ///
    /// It may panic if there are too many pairs in the PodMap already. Pay attention,
    /// it panics only in the `debug` mode. In the `release` mode, you are going to get
    /// **undefined behavior**. This is done for the sake of performance, in order to
    /// avoid a repetitive check for the boundary condition on every `insert()`.
    ///
    /// # Safety
    ///
    /// Calling this method to add a new key-value pair when the [`PodMap`] is already
    /// full is undefined behavior instead of panic. So you need to make sure that
    /// the PodMap is not full before calling.
    #[inline]
    pub unsafe fn insert_unchecked(&mut self, k: K, v: V) -> Option<V> {
        let (_, existing_pair) = self.insert_i(k, v);
        existing_pair.map(|(_, v)| v)
    }

    /// The core insert logic, which is used for `insert_unchecked()`, as it will
    /// disable the bound check (`debug_assert!`) in `release` mode.
    pub(crate) fn insert_i(&mut self, k: K, v: V) -> (usize, Option<(K, V)>) {
        let mut target = self.len;
        let mut i = 0;
        let mut existing_pair = None;
        loop {
            if i == self.len {
                core::debug_assert!(target < N, "No more key-value slot available in the PodMap");
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
            (i, Some(replace(pair, (k, v))))
        } else {
            let i = self.len;
            // just for panic msg in debug mode, not the main bound check
            core::debug_assert!(i < N, "No more key-value slot available in the PodMap");
            self.pairs[i].write((k, v)); // main bound check here but it's ok (not hotspot)
            self.len += 1;
            (i, None)
        }
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

    /// Removes a key from the PodMap, returning the stored key and value if the
    /// key was previously in the PodMap.
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

#[cfg(test)]
mod tests {

    use super::PodMap;

    #[test]
    fn insert_and_check_length() {
        let mut m: PodMap<u8, i32, 10> = PodMap::new();
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(1, m.len());
        assert_eq!(m.insert(2, 16), None);
        assert_eq!(2, m.len());
        assert_eq!(m.insert(1, 16), Some(42));
        assert_eq!(2, m.len());
    }

    #[test]
    fn overwrites_keys() {
        let mut m: PodMap<i32, i32, 1> = PodMap::new();
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.insert(1, 42), Some(42));
        assert_eq!(1, m.len());
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn cant_write_into_empty_pod_map() {
        let mut m: PodMap<i32, i32, 0> = PodMap::new();
        assert_eq!(m.insert(1, 42), None);
    }

    #[test]
    fn empty_length() {
        let m: PodMap<u32, u32, 10> = PodMap::new();
        assert_eq!(0, m.len());
    }

    #[test]
    fn is_empty_check() {
        let mut m: PodMap<u32, u32, 10> = PodMap::new();
        assert!(m.is_empty());
        assert_eq!(m.insert(42, 42), None);
        assert!(!m.is_empty());
    }

    #[test]
    fn insert_and_gets() {
        let mut m: PodMap<u8, i32, 10> = PodMap::new();
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.insert(2, 16), None);
        assert_eq!(16, *m.get(&2).unwrap());
    }

    #[test]
    fn insert_and_gets_mut() {
        let mut m: PodMap<i32, [i32; 3], 10> = PodMap::new();
        assert_eq!(m.insert(42, [1, 2, 3]), None);
        let a = m.get_mut(&42).unwrap();
        a[0] = 500;
        assert_eq!(500, m.get(&42).unwrap()[0]);
    }

    #[test]
    fn checks_key() {
        let mut m: PodMap<u8, i32, 10> = PodMap::new();
        assert_eq!(m.insert(1, 42), None);
        assert!(m.contains_key(&1));
        assert!(!m.contains_key(&3));
    }

    #[test]
    fn gets_missing_key() {
        let mut m: PodMap<u8, i32, 10> = PodMap::new();
        assert_eq!(m.insert(1, 42), None);
        assert!(m.get(&2).is_none());
    }

    #[test]
    fn mut_gets_missing_key() {
        let mut m: PodMap<u8, i32, 10> = PodMap::new();
        assert_eq!(m.insert(1, 42), None);
        assert!(m.get_mut(&2).is_none());
    }

    #[test]
    fn removes_simple_pair() {
        let mut m: PodMap<u8, i32, 10> = PodMap::new();
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.remove(&1), Some(42));
        assert_eq!(m.remove(&3), None);
        assert!(m.get(&1).is_none());
    }

    #[cfg(test)]
    #[derive(Clone, PartialEq, Debug)]
    struct Foo {
        v: [u32; 3],
    }

    #[test]
    fn insert_struct() {
        let mut m: PodMap<u8, Foo, 8> = PodMap::new();
        let foo = Foo { v: [1, 2, 100] };
        assert_eq!(m.insert(1, foo), None);
        assert_eq!(100, m.into_iter().next().unwrap().1.v[2]);
    }

    #[cfg(test)]
    #[derive(Clone, PartialEq, Debug)]
    struct Composite {
        r: PodMap<u8, u8, 1>,
    }

    #[test]
    fn insert_composite() {
        let mut m: PodMap<u8, Composite, 8> = PodMap::new();
        let c = Composite { r: PodMap::new() };
        assert_eq!(m.insert(1, c), None);
        assert_eq!(0, m.into_iter().next().unwrap().1.r.len());
    }

    #[test]
    fn large_pod_map_in_heap() {
        let m: Box<PodMap<u64, [u64; 10], 10>> = Box::new(PodMap::new());
        assert_eq!(0, m.len());
    }

    #[test]
    fn clears_it_up() {
        let mut m: PodMap<u8, i32, 10> = PodMap::new();
        assert_eq!(m.insert(1, 42), None);
        m.clear();
        assert_eq!(0, m.len());
    }

    #[test]
    fn retain_test() {
        let vec: Vec<(i32, i32)> = (0..8).map(|x| (x, x * 10)).collect();
        let mut m: PodMap<i32, i32, 10> = PodMap::from_iter(vec);
        assert_eq!(m.len(), 8);
        m.retain(|&k, _| k < 6);
        assert_eq!(m.len(), 6);
        m.retain(|_, &v| v > 30);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn insert_many_and_remove() {
        let mut m: PodMap<usize, u64, 4> = PodMap::new();
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
        let mut m: PodMap<[u8;3], i32, 10> = PodMap::new();
        let k = b"key";
        assert_eq!(m.insert(k.clone(), 42), None);
        assert_eq!(m.get_key_value(k), Some((k, &42)));
        assert!(m.contains_key(k));
    }

    #[test]
    fn get_absent_key_value() {
        let mut m: PodMap<u8, i32, 10> = PodMap::new();
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.get_key_value(&2), None);
    }

    #[test]
    fn remove_entry_present() {
        let mut m: PodMap<[u8;3], i32, 10> = PodMap::new();
        let k = b"key";
        assert_eq!(m.insert(k.clone(), 42), None);
        assert_eq!(m.remove_entry(k), Some((k.clone(), 42)));
        assert!(!m.contains_key(k));
    }

    #[test]
    fn remove_entry_absent() {
        let mut m: PodMap<u8, i32, 10> = PodMap::new();
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.remove_entry(&2), None);
    }

    #[test]
    fn drop_removed_entry() {
        use std::rc::Rc;
        let mut m: PodMap<(), Rc<()>, 8> = PodMap::new();
        let v = Rc::new(());
        assert_eq!(m.insert((), Rc::clone(&v)), None);
        assert_eq!(Rc::strong_count(&v), 2);
        assert_eq!(m.remove_entry(&()), Some(((), Rc::clone(&v))));
        assert_eq!(Rc::strong_count(&v), 1);
    }

    #[test]
    fn insert_after_remove() {
        let mut m: PodMap<_, _, 1> = PodMap::new();
        assert_eq!(m.insert(1, 2), None);
        assert_eq!(m.remove(&1), Some(2));
        assert_eq!(m.insert(1, 3), None);
    }

    #[test]
    fn insert_drop_duplicate() {
        use std::rc::Rc;
        let mut m: PodMap<_, _, 1> = PodMap::new();
        let v = Rc::new(());
        assert_eq!(m.insert((), Rc::clone(&v)), None);
        assert_eq!(Rc::strong_count(&v), 2);
        assert_eq!(m.insert((), Rc::clone(&v)), Some(Rc::clone(&v)));
        assert_eq!(Rc::strong_count(&v), 2);
    }

    #[test]
    fn insert_duplicate_after_remove() {
        let mut m: PodMap<_, _, 2> = PodMap::new();
        assert_eq!(m.insert(1, 1), None);
        assert_eq!(m.insert(2, 2), None);
        assert_eq!(m.remove(&1), Some(1));
        assert_eq!(m.insert(2, 3), Some(2));
        assert_eq!(1, m.len());
        assert_eq!(3, m[&2]);
    }
}
