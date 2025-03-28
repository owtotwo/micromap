// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use bytemuck::Pod;

use super::PodMap;
use core::fmt::{self, Display, Formatter, Write};

impl<K: PartialEq + Pod + Display, V: Display, const N: usize> Display for PodMap<K, V, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut first = true;
        f.write_char('{')?;
        for (k, v) in self {
            if first {
                first = false;
            } else {
                f.write_str(", ")?;
            }
            k.fmt(f)?;
            f.write_str(": ")?;
            v.fmt(f)?;
        }
        f.write_char('}')?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn displays_map() {
        let mut m: PodMap<[u8; 3], i32, 10> = PodMap::new();
        m.insert(b"one".to_owned(), 42);
        m.insert(b"two".to_owned(), 16);
        assert_eq!(
            r#"{[111, 110, 101]: 42, [116, 119, 111]: 16}"#,
            format!("{:?}", m)
        );
    }
}
