// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use bytemuck::Pod;

use super::PodMap;
use core::mem::MaybeUninit;

impl<K: PartialEq + Pod, V, const N: usize> Default for PodMap<K, V, N> {
    /// Make a default empty [`PodMap`].
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialEq + Pod, V, const N: usize> PodMap<K, V, N> {
    /// Make it.
    ///
    /// The size of the PodMap is defined by the generic argument. For example,
    /// this is how you make a PodMap of four key-values pairs:
    #[inline]
    #[must_use]
    #[allow(clippy::uninit_assumed_init)]
    pub const fn new() -> Self {
        Self {
            len: 0,
            pairs: [const { MaybeUninit::uninit() }; N],
        }
    }
}

impl<K: PartialEq + Pod, V, const N: usize> Drop for PodMap<K, V, N> {
    fn drop(&mut self) {
        for i in 0..self.len {
            unsafe { self.item_drop(i) };
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn makes_default_map() {
        let m: PodMap<u8, u8, 8> = PodMap::default();
        assert_eq!(0, m.len());
    }

    #[test]
    fn makes_new_map() {
        let m: PodMap<u8, u8, 8> = PodMap::new();
        assert_eq!(0, m.len());
    }

    #[test]
    fn drops_correctly() {
        let _m: PodMap<u8, Vec<u8>, 8> = PodMap::new();
    }

    #[test]
    fn drops_keys() {
        use std::rc::Rc;
        let mut m: PodMap<u8, Rc<()>, 8> = PodMap::new();
        let v1 = Rc::new(());
        assert_eq!(Rc::strong_count(&v1), 1);
        let v2 = Rc::new(());
        assert_eq!(Rc::strong_count(&v2), 1);

        m.insert(1, Rc::clone(&v1));
        assert_eq!(Rc::strong_count(&v1), 2);
        m.insert(2, Rc::clone(&v1));
        assert_eq!(Rc::strong_count(&v1), 3);
        m.insert(1, Rc::clone(&v2));
        assert_eq!(Rc::strong_count(&v1), 2);

        drop(m);
        assert_eq!(Rc::strong_count(&v1), 1);
    }

    #[test]
    fn drops_values() {
        use std::rc::Rc;
        let mut m: PodMap<(), Rc<()>, 8> = PodMap::new();
        let v = Rc::new(());
        m.insert((), Rc::clone(&v));
        drop(m);
        assert_eq!(Rc::strong_count(&v), 1);
    }
}
