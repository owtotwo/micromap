// SPDX-FileCopyrightText: Copyright (c) 2023-2025 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use micromap::Map;

pub fn insert_benchmark(c: &mut Criterion) {
    c.bench_function("insert_same", |b| {
        let mut m: Map<u64, u64, 64> = Map::new();
        b.iter(|| {
            for i in 0..1000 {
                black_box(m.insert(8, i));
            }
        });
    });

    c.bench_function("insert_different", |b| {
        const CAP: usize = 64;
        let mut m: Map<usize, u64, CAP> = Map::new();
        b.iter(|| {
            for i in 0..CAP {
                black_box(m.insert(i, 256));
            }
        });
    });

    c.bench_function("insert_and_remove", |b| {
        const CAP: usize = 64;
        let mut m: Map<usize, u64, CAP> = Map::new();
        b.iter(|| {
            for i in 0..CAP {
                black_box(m.insert(i, 256));
                black_box(m.remove(&i));
            }
        });
    });
}

pub fn length_benchmark(c: &mut Criterion) {
    c.bench_function("straight_length", |b| {
        let mut m: Map<u64, u64, 64> = Map::new();
        for i in 0..1000 {
            m.insert(8, i);
        }
        b.iter(|| {
            for _ in 0..1000 {
                black_box(m.len());
            }
        });
    });

    c.bench_function("fragmented_length", |b| {
        let mut m: Map<u64, u64, 64> = Map::new();
        for i in 0..1000 {
            m.insert(8, i);
        }
        for i in 0..1000 {
            if i % 7 != 0 {
                m.remove(&i);
            }
        }
        b.iter(|| {
            for _ in 0..1000 {
                black_box(m.len());
            }
        });
    });
}

#[ignore]
pub fn insert_exist_kv_in_diff_slot(c: &mut Criterion) {
    c.bench_function("insert_index_0_slot", |b| {
        let mut m = Map::<u32, char, 64>::from_iter([(0, 'a')]);
        b.iter(|| black_box(m.insert(black_box(0), black_box('a'))));
    });

    c.bench_function("insert_index_3_slot_as_tail_in_small_map", |b| {
        let mut m = Map::<u32, char, 4>::from_iter([(0, 'a'), (1, 'b'), (2, 'c'), (3, 'd')]);
        b.iter(|| black_box(m.insert(black_box(3), black_box('d'))));
    });

    c.bench_function("insert_index_3_slot", |b| {
        let mut m = Map::<u32, char, 64>::from_iter([(0, 'a'), (1, 'b'), (2, 'c'), (3, 'd')]);
        b.iter(|| black_box(m.insert(black_box(3), black_box('d'))));
    });

    c.bench_function("insert_index_3_slot_in_large_map", |b| {
        let mut m = Map::<u32, char, 4096>::from_iter([(0, 'a'), (1, 'b'), (2, 'c'), (3, 'd')]);
        b.iter(|| black_box(m.insert(black_box(3), black_box('d'))));
    });

    c.bench_function("insert_index_3_slot_when_full", |b| {
        let mut m = Map::<u32, char, 64>::new();
        (0..64).for_each(|i| {
            m.insert(i, 'x');
        });
        b.iter(|| black_box(m.insert(black_box(3), black_box('d'))));
    });

    c.bench_function("insert_index_30_slot", |b| {
        let mut m = Map::<u32, char, 64>::new();
        (0..31).for_each(|i| {
            m.insert(i, 'x');
        });
        b.iter(|| black_box(m.insert(black_box(30), black_box('y'))));
    });

    c.bench_function("insert_0_to_63_slot", |b| {
        let mut m = Map::<u32, char, 64>::new();
        (0..64).for_each(|i| {
            m.insert(i, 'x');
        });
        b.iter(|| {
            (0..64).for_each(|i| {
                black_box(m.insert(black_box(i), black_box('y')));
            })
        });
    });

    c.bench_function("insert_63_to_0_slot", |b| {
        let mut m = Map::<u32, char, 64>::new();
        (0..64).for_each(|i| {
            m.insert(i, 'x');
        });
        b.iter(|| {
            (0..64).rev().for_each(|i| {
                black_box(m.insert(black_box(i), black_box('y')));
            })
        });
    });
}

criterion_group!(
    benches,
    insert_benchmark,
    length_benchmark,
    // insert_exist_kv_in_diff_slot
);
criterion_main!(benches);
