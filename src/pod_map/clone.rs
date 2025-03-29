// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use bytemuck::Pod;

use super::PodMap;

impl<K: Clone + PartialEq + Pod, V: Clone, const N: usize> Clone for PodMap<K, V, N> {
    fn clone(&self) -> Self {
        let mut pm = Self::new();
        pm.len = self.len;
        pm.keys[..self.len]
            .iter_mut()
            .zip(self.keys[..self.len].iter())
            .for_each(|(k1, k2)| *k1 = *k2);
        pm.values[..self.len]
            .iter_mut()
            .zip(self.values[..self.len].iter())
            .for_each(|(v1, v2)| {
                let v1 = unsafe { v1.assume_init_mut() };
                let v2 = unsafe { v2.assume_init_ref() };
                *v1 = v2.clone();
            });
        pm
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn map_can_be_cloned() {
        let mut m: PodMap<u8, u8, 16> = PodMap::new();
        m.insert(0, 42);
        assert_eq!(42, *m.clone().get(&0).unwrap());
    }

    #[test]
    fn empty_map_can_be_cloned() {
        let m: PodMap<u8, u8, 0> = PodMap::new();
        assert!(m.clone().is_empty());
    }
}
