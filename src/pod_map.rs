// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! This is a simpler and faster alternative implementation of the standard `HashMap`.
//!
//! For example, here is how a map with a few keys can be created:
//!
//! ```
//! use micromap::PodMap as Map;
//! let mut m : Map<u64, &str, 10> = Map::new(); // u64 has Pod trait
//! m.insert(1, "Hello, world!");
//! m.insert(2, "Good bye!");
//! assert_eq!(2, m.len());
//! ```
//!
//! Creating a [`PodMap`] requires knowing the maximum size of it, upfront. This is
//! what the third type argument `10` is for, in the example above. The array
//! will have exactly ten elements. An attempt to add an 11th element will lead
//! to a panic.

mod clone;
mod ctors;
mod debug;
mod display;
mod drain;
mod entry;
mod eq;
mod from;
mod index;
mod into;
mod iterators;
mod keys;
mod map;
#[cfg(feature = "serde")]
mod serialization;
mod values;

use core::mem::MaybeUninit;

use bytemuck::Pod;


/// A special version of [`crate::Map`], which Key type has [`Pod`] trait.
///
/// For example, this is how you make a map, which is allocated on stack and is capable of storing
/// up to eight key-values pairs:
///
/// ```
/// let mut m : micromap::Map<u64, Vec<char>, 8> = micromap::PodMap::new(); // u64 is Pod type
/// m.insert(1, vec!['H', 'e', 'l', 'l', 'o']);
/// m.insert(2, vec!['W', 'o', 'r', 'l', 'd', '!']);
/// assert_eq!(2, m.len());
/// ```
pub struct Map<K: PartialEq, V, const N: usize> {
    /// The next available pair in the array.
    len: usize,
    /// The bit mask of the keys.
    bits: (K, K),
    /// The fixed-size array of key-value pairs.
    pairs: [MaybeUninit<(K, V)>; N],
}
pub struct PodMap<K: PartialEq + Pod, V, const N: usize> {
    /// The next available pair in the array.
    len: usize,
    /// The bit mask of the keys.
    bits: (K, K),
    /// The fixed-size array of key-value pairs.
    pairs: [MaybeUninit<(K, V)>; N],
}

/// Iterator over the [`PodMap`].
#[repr(transparent)]
pub struct Iter<'a, K, V> {
    iter: core::slice::Iter<'a, MaybeUninit<(K, V)>>,
}

/// Mutable Iterator over the [`PodMap`].
#[repr(transparent)]
pub struct IterMut<'a, K, V> {
    iter: core::slice::IterMut<'a, MaybeUninit<(K, V)>>,
}

/// Into-iterator over the [`PodMap`].
#[repr(transparent)]
pub struct IntoIter<K: PartialEq + Pod, V, const N: usize> {
    map: PodMap<K, V, N>,
}

/// An iterator over the values of the [`PodMap`].
#[repr(transparent)]
pub struct Values<'a, K, V> {
    iter: Iter<'a, K, V>,
}

/// Mutable iterator over the values of the [`PodMap`].
#[repr(transparent)]
pub struct ValuesMut<'a, K, V> {
    iter: IterMut<'a, K, V>,
}

/// Consuming iterator over the values of the [`PodMap`].
#[repr(transparent)]
pub struct IntoValues<K: PartialEq + Pod, V, const N: usize> {
    iter: IntoIter<K, V, N>,
}

/// A read-only iterator over the keys of the [`PodMap`].
#[repr(transparent)]
pub struct Keys<'a, K, V> {
    iter: Iter<'a, K, V>,
}

/// Consuming iterator over the keys of the [`PodMap`].
#[repr(transparent)]
pub struct IntoKeys<K: PartialEq + Pod, V, const N: usize> {
    iter: IntoIter<K, V, N>,
}

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`PodMap`].
///
/// [`entry`]: Map::entry
pub enum Entry<'a, K: PartialEq + Pod, V, const N: usize> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V, N>),

    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V, N>),
}

/// A view into an occupied entry in a `PodMap`.
/// It is part of the [`Entry`] enum.
pub struct OccupiedEntry<'a, K: PartialEq + Pod, V, const N: usize> {
    index: usize,
    table: &'a mut PodMap<K, V, N>,
}

/// A view into a vacant entry in a `PodMap`.
/// It is part of the [`Entry`] enum.
pub struct VacantEntry<'a, K: PartialEq + Pod, V, const N: usize> {
    key: K,
    table: &'a mut PodMap<K, V, N>,
}

/// A draining iterator over the entries of a `PodMap`.
///
/// This struct is created by the drain method on `PodMap`. See its documentation for more.
#[deny(clippy::needless_lifetimes)]
pub struct Drain<'a, K, V> {
    iter: core::slice::IterMut<'a, MaybeUninit<(K, V)>>,
}
