// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use bytemuck::Pod;

use super::PodMap;
use core::borrow::Borrow;
use core::ops::{Index, IndexMut};

impl<K: PartialEq + Pod + Borrow<Q>, Q: PartialEq + ?Sized, V, const N: usize> Index<&Q>
    for PodMap<K, V, N>
{
    type Output = V;

    #[inline]
    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("No entry found for the key")
    }
}

impl<K: PartialEq + Pod + Borrow<Q>, Q: PartialEq + ?Sized, V, const N: usize> IndexMut<&Q>
    for PodMap<K, V, N>
{
    #[inline]
    fn index_mut(&mut self, key: &Q) -> &mut V {
        self.get_mut(key).expect("No entry found for the key")
    }
}

#[cfg(test)]
mod tests {

    use bytemuck::Zeroable;

    use super::*;

    #[test]
    fn index() {
        let mut m: PodMap<[u8; 5], i32, 10> = PodMap::new();
        m.insert(b"first".to_owned(), 42);
        assert_eq!(m[b"first"], 42);
    }

    #[test]
    fn index_mut() {
        let mut m: PodMap<[u8; 5], i32, 10> = PodMap::new();
        m.insert(b"first".to_owned(), 42);
        m[b"first"] += 10;
        assert_eq!(m[b"first"], 52);
    }

    #[test]
    #[should_panic]
    fn wrong_index() -> () {
        let mut m: PodMap<[u8; 5], i32, 10> = PodMap::new();
        m.insert(b"first".to_owned(), 42);
        assert_eq!(m[b"secnd"], 42);
    }

    #[cfg(test)]
    #[derive(PartialEq, Clone, Copy, Zeroable, Pod)]
    #[repr(transparent)]
    struct Container {
        pub t: i32,
    }

    #[cfg(test)]
    impl Borrow<i32> for Container {
        fn borrow(&self) -> &i32 {
            &self.t
        }
    }

    #[test]
    fn index_by_borrow() {
        let mut m: PodMap<Container, i32, 10> = PodMap::new();
        m.insert(Container { t: 10 }, 42);
        assert_eq!(m[&10], 42);
    }
}
