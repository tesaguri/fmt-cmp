//! Traits and utilities for lexicographically comparing values in their `Display` representations.

#![doc(html_root_url = "https://docs.rs/fmt-cmp/0.1.0")]
#![cfg_attr(not(feature = "std"), no_std)]
// Features.
#![cfg_attr(fmt_cmp_semver_exempt, feature(min_specialization))]
#![cfg_attr(fmt_cmp_semver_exempt, feature(int_log))]
// Lints.
#![warn(missing_docs)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
extern crate core as std;
#[cfg(feature = "std")]
extern crate std as alloc;

pub mod cmp;
pub mod int;

mod traits;

pub use self::cmp::{cmp, eq, hash, Cmp};
pub use self::int::{cmp_dec, cmp_int};
pub use self::traits::{FmtEq, FmtOrd};
