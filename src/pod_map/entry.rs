// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use bytemuck::Pod;

use super::{Entry, OccupiedEntry, VacantEntry};
use core::mem;

impl<'a, K: PartialEq + Pod, V, const N: usize> Entry<'a, K, V, N> {
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }

    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
        }
    }

    pub fn key(&self) -> &K {
        match self {
            Entry::Occupied(entry) => entry.key(),
            Entry::Vacant(entry) => entry.key(),
        }
    }

    #[must_use]
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }
}

impl<'a, K: PartialEq + Pod, V: Default, const N: usize> Entry<'a, K, V, N> {
    pub fn or_default(self) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(V::default()),
        }
    }
}

impl<'a, K: PartialEq + Pod, V, const N: usize> OccupiedEntry<'a, K, V, N> {
    #[must_use]
    pub fn key(&self) -> &K {
        unsafe { self.table.item_ref(self.index).0 }
    }

    #[must_use]
    pub fn remove_entry(self) -> (K, V) {
        unsafe { self.table.remove_index_read(self.index) }
    }

    #[must_use]
    pub fn get(&self) -> &V {
        unsafe { self.table.item_ref(self.index).1 }
    }

    pub fn get_mut(&mut self) -> &mut V {
        unsafe { self.table.item_mut(self.index).1 }
    }

    #[must_use]
    pub fn into_mut(self) -> &'a mut V {
        unsafe { self.table.item_mut(self.index).1 }
    }

    pub fn insert(&mut self, value: V) -> V {
        mem::replace(self.get_mut(), value)
    }

    #[must_use]
    pub fn remove(self) -> V {
        unsafe { self.table.remove_index_read(self.index).1 }
    }
}

impl<'a, K: PartialEq + Pod, V, const N: usize> VacantEntry<'a, K, V, N> {
    pub const fn key(&self) -> &K {
        &self.key
    }

    pub const fn into_key(self) -> K {
        self.key
    }

    pub fn insert(self, value: V) -> &'a mut V {
        let (index, _) = self.table.insert_i(self.key, value);
        unsafe { self.table.item_mut(index).1 }
    }
}

#[cfg(test)]
mod tests {

    use bytemuck::{Pod, Zeroable};

    use super::Entry;
    use crate::PodMap;

    #[derive(Debug, PartialEq, Clone, Copy, Zeroable, Pod)]
    #[repr(transparent)]
    struct PodChar(u8);

    #[test]
    fn various() {
        let mut m: PodMap<PodChar, u8, 10> = PodMap::from_iter([
            (PodChar(b'a'), 97),
            (PodChar(b'd'), 100),
            (PodChar(b'c'), 99),
            (PodChar(b'b'), 98),
        ]);
        let e: Entry<'_, PodChar, u8, 10> = m.entry(PodChar(b'c'));
        assert_eq!(e.key(), &PodChar(b'c'));
        m.entry(PodChar(b'e')).or_insert(b'e');
        m.entry(PodChar(b'e')).or_insert(b'e');
        assert_eq!(
            *m.entry(PodChar(b'e')).and_modify(|v| *v = 42).or_default(),
            42
        );
        assert_eq!(m.entry(PodChar(b'g')).key(), &PodChar(b'g'));
        assert_eq!(
            *m.entry(PodChar(b'g')).and_modify(|v| *v = 42).or_default(),
            u8::default()
        );
        let value = if let Entry::Occupied(mut entry) = m.entry(PodChar(b'e')) {
            let value = *entry.get();
            assert_eq!(value, 42);

            *entry.get_mut() = b'E';
            assert_eq!(*entry.get(), 69);
            let e = entry.into_mut();
            *e = b'e';
            value
        } else {
            100
        };
        assert_eq!(*m.entry(PodChar(b'f')).or_insert_with(|| value + 1), 43); // _ -> 43
        assert_eq!(*m.entry(PodChar(b'f')).or_insert_with(|| value + 2), 43); // no change
        assert_eq!(m.remove_entry(&PodChar(b'f')), Some((PodChar(b'f'), 43))); // 43 -> _
        assert_eq!(
            *m.entry(PodChar(b'f')).or_insert_with_key(|&key| key.0),
            102
        ); // _ -> 102
        assert_eq!(*m.entry(PodChar(b'f')).or_insert_with_key(|&_| 255), 102); // no change
        if let Entry::Occupied(entry) = m.entry(PodChar(b'e')) {
            assert_eq!(entry.remove(), 101);
        }
        if let Entry::Vacant(entry) = m.entry(PodChar(b'e')) {
            assert_eq!(entry.key(), &PodChar(b'e'));
            assert_eq!(entry.into_key(), PodChar(b'e'));
        }
        if let Entry::Vacant(entry) = m.entry(PodChar(b'e')) {
            assert_eq!(entry.key(), &PodChar(b'e'));
            entry.insert(b'e');
        }
        if let Entry::Occupied(mut entry) = m.entry(PodChar(b'e')) {
            entry.insert(b'E');
            assert_eq!(entry.remove_entry(), (PodChar(b'e'), b'E'));
        }
    }
}
