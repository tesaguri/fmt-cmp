# `fmt-cmp`

[![GitHub Actions (CI)](https://github.com/tesaguri/fmt-cmp/workflows/CI/badge.svg)](https://github.com/tesaguri/fmt-cmp/actions)
[![crates.io](https://img.shields.io/crates/v/fmt-cmp.svg)](https://crates.io/crates/fmt-cmp)
[![docs.rs](https://docs.rs/fmt-cmp/badge.svg)](https://docs.rs/fmt-cmp/)
![Rust 1.41.0+](https://img.shields.io/badge/rust-1.41.0%2B-blue.svg)

A Rust library for lexicographically comparing values in their `Display`
representations.

The utilities provided by this library gives the same results as comparing
values after applying `to_string()`, but they never allocate on the heap memory.

<!-- TODO: Overview -->

## Examples

Compare digits of numbers:

```rust
assert!(fmt_cmp::eq(f64::NAN, f64::NAN)); // `"NaN" == "NaN"`
assert!(fmt_cmp::cmp(&42, &240).is_gt()); // `"42" > "240"`
```

Sorting integers _lexicographically_:

```rust
use std::collections::BTreeSet;

use fmt_cmp::Cmp as FmtCmp;

let mut values: BTreeSet<FmtCmp<u32>> = (1..=10).map(FmtCmp).collect();
assert!(values
   .into_iter()
   .map(|cmp| cmp.0)
   .eq([1, 10, 2, 3, 4, 5, 6, 7, 8, 9]));
```

## License

Copyright (c) 2021 Daiki "tesaguri" Mizukami

This project is licensed under either of:

- The Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>), or
- The MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.
