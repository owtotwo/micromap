// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use core::mem::MaybeUninit;

use bytemuck::Pod;

use super::PodMap;

impl<K: Clone + PartialEq + Pod, V: Clone, const N: usize> Clone for PodMap<K, V, N> {
    fn clone(&self) -> Self {
        let mut pm = PodMap {
            len: self.len,
            bits: self.bits.clone(),
            pairs: [const { MaybeUninit::uninit() }; N],
        };
        pm.pairs[..self.len]
            .iter_mut()
            .zip(self.pairs[..self.len].iter())
            .for_each(|(p1, p2)| {
                let (k1, v1) = unsafe { p1.assume_init_mut() };
                let (k2, v2) = unsafe { p2.assume_init_ref() };
                *k1 = k2.clone();
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
