// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use bytemuck::Pod;

use super::PodMap;

impl<K: PartialEq + Pod, V: PartialEq, const N: usize, const M: usize> PartialEq<PodMap<K, V, M>>
    for PodMap<K, V, N>
{
    /// Two maps can be compared. (The capacity does not affect comparison.)
    ///
    /// For example:
    ///
    /// ```
    /// let mut m1: micromap::PodMap<u8, i32, 5> = micromap::PodMap::new();
    /// let mut m2: micromap::PodMap<u8, i32, 10> = micromap::PodMap::new();
    /// m1.insert(1, 42);
    /// m2.insert(1, 42);
    ///
    /// assert_eq!(m1, m2);
    /// // two maps with different order of key-value pairs are still equal:
    /// m1.insert(2, 1);
    /// m1.insert(3, 16);
    /// m2.insert(3, 16);
    /// m2.insert(2, 1);
    ///
    /// assert_eq!(m1, m2);
    /// ```
    #[inline]
    fn eq(&self, other: &PodMap<K, V, M>) -> bool {
        self.len() == other.len() && self.iter().all(|(k, v)| other.get(k) == Some(v))
    }
}

impl<K: Eq + Pod, V: Eq, const N: usize> Eq for PodMap<K, V, N> {}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn compares_two_maps() {
        let mut m1: PodMap<[u8; 5], i32, 5> = PodMap::new();
        m1.insert(b"first".to_owned(), 42);
        let mut m2: PodMap<[u8; 5], i32, 5> = PodMap::new();
        m2.insert(b"first".to_owned(), 42);
        assert!(m1.eq(&m2));
    }

    #[test]
    fn compares_two_diff_cap_maps() {
        let mut m1: PodMap<u8, i32, 3> = PodMap::from([(b'a', 97), (b'b', 98), (b'c', 99)]);
        let mut m2: PodMap<u8, i32, 4> =
            PodMap::from([(b'c', 99), (b'c', 99), (b'c', 99), (b'b', 98)]);
        m2.insert(b'a', 97);
        assert!(m1.eq(&m2));
        m1.remove(&b'c');
        assert!(m1.ne(&m2));
    }
}
