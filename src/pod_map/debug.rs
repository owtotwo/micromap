// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use bytemuck::Pod;

use super::PodMap;
use core::fmt::{self, Debug, Formatter};

impl<K: PartialEq + Pod + Debug, V: Debug, const N: usize> Debug for PodMap<K, V, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn debugs_map() {
        let mut m: PodMap<[u8;3], i32, 10> = PodMap::new();
        m.insert(b"one".to_owned(), 42);
        m.insert(b"two".to_owned(), 16);
        assert_eq!(r#"{"one": 42, "two": 16}"#, format!("{:?}", m));
    }

    #[test]
    fn debug_alternate_map() {
        let mut m: PodMap<[u8;3], i32, 10> = PodMap::new();
        m.insert(b"one".to_owned(), 42);
        m.insert(b"two".to_owned(), 16);
        assert_eq!(
            r#"{
    "one": 42,
    "two": 16,
}"#,
            format!("{:#?}", m)
        );
    }
}
