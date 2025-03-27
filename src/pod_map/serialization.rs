// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use super::PodMap;
use bytemuck::Pod;
use core::fmt::Formatter;
use core::marker::PhantomData;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl<K: PartialEq + Pod + Serialize, V: Serialize, const N: usize> Serialize for PodMap<K, V, N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut PodMap = serializer.serialize_map(Some(self.len()))?;
        for (a, v) in self {
            PodMap.serialize_entry(a, v)?;
        }
        PodMap.end()
    }
}

struct Vi<K, V, const N: usize>(PhantomData<K>, PhantomData<V>);

impl<'de, K: PartialEq + Pod + Deserialize<'de>, V: Deserialize<'de>, const N: usize> Visitor<'de>
    for Vi<K, V, N>
{
    type Value = PodMap<K, V, N>;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("a PodMap")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut m: Self::Value = PodMap::new();
        while let Some((key, value)) = access.next_entry()? {
            m.insert(key, value);
        }
        Ok(m)
    }
}

impl<'de, K: PartialEq + Pod + Deserialize<'de>, V: Deserialize<'de>, const N: usize>
    Deserialize<'de> for PodMap<K, V, N>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(Vi(PhantomData, PhantomData))
    }
}

#[cfg(test)]
mod tests {

    use super::PodMap;
    use bincode::serde::{decode_from_slice, encode_into_slice};

    #[test]
    fn serialize_and_deserialize() {
        let config = bincode::config::legacy();
        let mut before: PodMap<u8, u8, 8> = PodMap::new();
        before.insert(1, 42);
        let mut bytes: [u8; 1024] = [0; 1024];
        let len = encode_into_slice(&before, &mut bytes, config).unwrap();
        let bytes = &bytes[..len];
        println!("bytes: {:?}", bytes);
        let (after, read_len): (PodMap<u8, u8, 8>, usize) =
            decode_from_slice(&bytes, config).unwrap();
        assert_eq!(42, after.into_iter().next().unwrap().1);
        assert_eq!(bytes.len(), read_len);
    }

    #[test]
    fn empty_map_serde() {
        let config = bincode::config::legacy();
        let before: PodMap<u8, u8, 8> = PodMap::new();
        let mut bytes: [u8; 1024] = [0; 1024];
        let len = encode_into_slice(&before, &mut bytes, config).unwrap();
        let bytes = &bytes[..len];
        let (after, read_len): (PodMap<u8, u8, 8>, usize) =
            decode_from_slice(&bytes, config).unwrap();
        assert!(after.is_empty());
        assert_eq!(bytes.len(), read_len);
    }
}
