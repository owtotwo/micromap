// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::SetDrain;
use core::iter::FusedIterator;

impl<K> Iterator for SetDrain<'_, K> {
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, ())| k)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.iter.len(), Some(self.iter.len()))
    }
}

impl<K> ExactSizeIterator for SetDrain<'_, K> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K> FusedIterator for SetDrain<'_, K> {}
