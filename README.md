# The Fastest Linear Map in Rust

[![cargo](https://github.com/owtotwo/micromap/actions/workflows/cargo.yml/badge.svg)](https://github.com/owtotwo/micromap/actions/workflows/cargo.yml)
[![crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange.svg)](https://crates.io/crates/micromap)
[![docs.rs](https://img.shields.io/docsrs/micromap)](https://docs.rs/micromap/latest/micromap/)
[![MSRV](https://img.shields.io/badge/MSRV-1.79-ffc832)](https://blog.rust-lang.org/2024/06/13/Rust-1.79.0.html)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](https://github.com/owtotwo/micromap/blob/master/LICENSE.txt)
[![codecov](https://codecov.io/gh/owtotwo/micromap/branch/master/graph/badge.svg)](https://codecov.io/gh/owtotwo/micromap)
[![Hits-of-Code](https://hitsofcode.com/github/owtotwo/micromap)](https://hitsofcode.com/view/github/owtotwo/micromap)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fyegor256%2Fmicromap.svg?type=shield&issueType=license)](https://app.fossa.com/projects/git%2Bgithub.com%2Fyegor256%2Fmicromap?ref=badge_shield&issueType=license)

A much faster alternative of
[`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html),
for very small maps.
It is also faster than
[FxHashMap](https://github.com/rust-lang/rustc-hash),
[hashbrown](https://github.com/rust-lang/hashbrown),
[ArrayMap](https://github.com/robjtede/tinymap),
[IndexMap](https://crates.io/crates/indexmap),
and _all_ others.
The smaller the map, the higher the performance.
It was observed that when a map contains more than 20 keys,
it may be better to use the standard
[`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html),
since the performance of `micromap::Map` _may_ start to degrade.
See the [benchmarking results](#benchmark) below.

**WELCOME**:
Not all functions that you might expect to have in a map are implemented.
I will appreciate if you contribute by implementing these
[missing functions](https://github.com/yegor256/micromap/issues).

First, add this to `Cargo.toml`:

```toml
[dependencies]
micromap = "0.1.0"
```

Then, use it like a standard hash map... well, almost:

```rust
use micromap::Map;
let mut m : Map<u64, &str, 10> = Map::new(); // allocation on stack
m.insert(1, "foo");
m.insert(2, "bar");
assert_eq!(2, m.len());
```

Pay attention, here the map is created with an extra generic argument `10`.
This is the total size of the map, which is allocated on stack when `::new()`
is called. Unlike `HashMap`, the `Map` doesn't use heap at all. If more than
ten keys will be added to the map, it will panic.

Read [the API documentation](https://docs.rs/micromap/latest/micromap/).
The struct
[`micromap::Map`](https://docs.rs/micromap/latest/micromap/struct.Map.html)
is designed to be as closely similar to
[`std::collections::HashMap`][std] as possible.

## Benchmark

There is a summary of a simple benchmark, where we compared `micromap::Map` with
a few other Rust maps, changing the total capacity of the map (horizontal axis).
We applied the same interactions
([`benchmark.rs`][rs])
to them and measured how fast they performed. In the following table,
the numbers over 1.0 indicate performance gain,
while the numbers below 1.0 demonstrate performance loss.

<!-- benchmark -->
| | 2 | 4 | 8 | 16 | 32 | 64 | 128 |
| --- | --: | --: | --: | --: | --: | --: | --: |
| `flurry::HashMap` | 349.49 | 77.18 | 39.93 | 20.84 | 11.53 | 5.90 | 3.05 |
| `hashbrown::HashMap` | 20.92 | 10.14 | 5.74 | 2.96 | 1.38 | 0.66 | 0.30 |
| `heapless::LinearMap` | 1.00 | 1.37 | 1.11 | 1.14 | 1.11 | 1.06 | 0.94 |
| `indexmap::IndexMap` | 16.32 | 13.76 | 8.73 | 5.09 | 2.73 | 1.37 | 0.69 |
| `linear_map::LinearMap` | 1.66 | 1.39 | 0.96 | 0.91 | 0.98 | 1.20 | 0.94 |
| `linked_hash_map::LinkedHashMap` | 32.12 | 18.91 | 11.16 | 6.47 | 3.44 | 1.73 | 0.89 |
| `litemap::LiteMap` | 1.66 | 2.02 | 5.90 | 3.61 | 1.79 | 1.05 | 0.63 |
| `micromap::Map` 👍 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |
| `nohash_hasher::BuildNoHashHasher` | 20.01 | 10.39 | 6.28 | 2.84 | 1.49 | 0.74 | 0.36 |
| `rustc_hash::FxHashMap` | 20.12 | 10.18 | 5.78 | 2.73 | 1.21 | 0.59 | 0.30 |
| `std::collections::BTreeMap` | 23.64 | 15.79 | 8.88 | 5.77 | 2.90 | 1.45 | 0.86 |
| `std::collections::HashMap` | 20.71 | 12.91 | 8.70 | 4.90 | 2.62 | 1.17 | 0.58 |
| `tinymap::array_map::ArrayMap` | 2.76 | 3.94 | 3.83 | 4.21 | 4.87 | 5.25 | 5.17 |

The experiment [was performed][action] on 29-06-2026.
There were 1000000 repetition cycles.
The entire benchmark took 273s.
Uname: 'Linux'.

<!-- benchmark -->

As you see, the highest performance gain was achieved for the maps that
were smaller than ten keys.
For the maps of just a few keys, the gain was enormous.

## MSRV (Minimum Supported Rust Version)

**`Rust 1.79`**

(Enabling some features will affect MSRV, the documentation will note it.)

## How to Contribute

First, install [Rust](https://www.rust-lang.org/tools/install), update to the
last version by `rustup update stable`, and then:

```bash
cargo test -vv
```

If everything goes well, fork repository, make changes, send us a
[pull request](https://www.yegor256.com/2014/04/15/github-guidelines.html).
We will review your changes and apply them to the `master` branch shortly,
provided they don't violate our quality standards. To avoid frustration,
before sending us your pull request please run `cargo test` again. Also,
run `cargo fmt` and `cargo clippy`.

Also, before you start making changes, run benchmarks:

```bash
cargo bench --bench bench
```

If you modified the comment docs, run this to check:

* Linux:

    ```bash
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features --no-deps
    ```

* Windows(PowerShell):

    ```PowerShell
    $env:RUSTDOCFLAGS="--cfg docsrs"; cargo +nightly doc --all-features --no-deps --open; Remove-Item Env:\RUSTDOCFLAGS
    ```

Then, after the changes you make, run it again. Compare the results.
If your changes
degrade performance, think twice before submitting a pull request.

About the **version change**, we follow the rules of this
[Cargo SemVer reference](https://doc.rust-lang.org/cargo/reference/semver.html)
. If your code has an impact on semver compatibility, such as
**breaking changes**, then you may also need to explicitly upgrade the version.
Because our project version uses a placeholder, you can
_add a hint note after the version number `0.0.0`_ in Cargo.toml
`package.version` to mark that you want to update the version, which we call
"version hint", as follows:

```toml
[package]
name = "micromap"
version = "0.0.0" # hint: 1.2.3
# ...
```

If no version change is required, do not add any comments after the version
number `0.0.0`.

[std]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
[rs]: ./blob/master/tests/benchmark.rs
[action]: ./actions/workflows/benchmark.yml
