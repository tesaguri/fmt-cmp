//! Lexicographic comparison utility for integers.

mod traits;

pub use self::traits::Integer;

use std::cmp::Ordering;

macro_rules! imp {
    ($lhs:expr, $rhs:expr, |$min:ident, $max:ident| $align:expr) => {{
        let (lhs, rhs) = ($lhs, $rhs);

        let (lhs, rhs, reversed) = if lhs.copy().lt(rhs.copy()) {
            (rhs, lhs, true)
        } else if lhs.copy().eq(rhs.copy()) {
            return Ordering::Equal;
        } else {
            (lhs, rhs, false)
        };

        // Align the number of digits to make numerical comparison equivalent to lexicographical
        // comparison. Since `'0' < '9' < 'A' < 'Z' (< 'a' < 'z')` holds, we don't need to
        // special-case radixes greater than 10.
        let lhs = match (lhs.copy(), rhs.copy()) {
            ($max, $min) => $align,
        };

        if lhs.lt(rhs) ^ reversed {
            Ordering::Less
        } else {
            // We've ruled out the case that the input `lhs` equals the input `rhs`, so if `lhs`
            // here equals `rhs`, it has been truncated and thus has been greater originally.
            Ordering::Greater
        }
    }};
}

/// Lexicographically compares the digits of two integers.
///
/// While being able to compare numbers in arbitrary radix, this is not optimized very well.
/// You should use [`cmp_dec`] for comparing in decimal representation or
/// <code>[fmt_cmp::cmp](crate::cmp())`(&format_args!("{:X}", lhs), &format_args!("{:X}", rhs))`</code>
/// for comparing in hexadecimal representation (`"{:o}"` for octal) instead.
///
/// When `radix == 1`, this will compare digits in the [unary system], i.e., will return the same
/// result as `lhs.cmp(&rhs)`.
///
/// When `radix > 36`, this will compare digits in a theoretical _base-`radix` system_, in which
/// the `radix`-th digit compares greater than the `(radix-1)`-th digit.
///
/// ## Panics
///
/// Panics if `radix == 0`.
///
/// ## Example
///
/// ```
/// assert!(fmt_cmp::cmp_int::<u32>(42, 3, 10).is_gt());
/// assert!(fmt_cmp::cmp_int::<u32>(24, 3, 10).is_lt());
///
/// assert!(fmt_cmp::cmp_int::<u32>(0x2a, 0x9, 16).is_lt());
/// assert!(fmt_cmp::cmp_int::<u32>(0xa2, 0x9, 16).is_gt());
/// ```
///
/// [unary system]: <https://en.wikipedia.org/wiki/Unary_numeral_system>
#[must_use]
pub fn cmp_int<T: Integer>(lhs: T, rhs: T, radix: u32) -> Ordering {
    if radix == 0 {
        panic!("`radix` must be greater than 0");
    }

    imp!(lhs, rhs, |min, max| max
        .copy()
        .invpow(radix, max.ilog(radix) - min.ilog(radix)))
}

/// Lexicographically compares the digits of two integers in their decimal representation.
///
/// This yields the same result as `lhs.to_string().cmp(&rhs.to_string())` without heap allocation.
///
/// ## Example
///
/// ```
/// assert!(fmt_cmp::cmp_dec::<u32>(42, 3).is_gt());
/// assert!(fmt_cmp::cmp_dec::<u32>(24, 3).is_lt());
/// ```
#[must_use]
pub fn cmp_dec<T: Integer>(lhs: T, rhs: T) -> Ordering {
    imp!(lhs, rhs, |min, max| max
        .copy()
        .invpow(10_u32, max.ilog10() - min.ilog10()))
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "alloc"))]
    extern crate alloc;

    use alloc::string::ToString;

    use super::*;

    #[test]
    fn matches_str_cmp() {
        #[track_caller]
        fn check<T: Copy + Integer + Ord + ToString>(lhs: T, rhs: T) {
            let expected = lhs.to_string().cmp(&rhs.to_string());
            assert_eq!(cmp_int(lhs, rhs, 10), expected);
            assert_eq!(cmp_int(rhs, lhs, 10), expected.reverse(), "reverse");
            assert_eq!(cmp_dec(lhs, rhs), expected, "dec");
            assert_eq!(cmp_dec(rhs, lhs), expected.reverse(), "dec,reverse");
            assert_eq!(cmp_int(lhs, rhs, 1), lhs.cmp(&rhs));
            assert_eq!(cmp_int(rhs, lhs, 1), rhs.cmp(&lhs));
        }

        // Both are 0.
        check(0_u64, 0_u64);

        // Either is 0.
        check(1_u64, 0_u64);
        check(10_u64, 0_u64);
        check(42_u64, 0_u64);

        // Single digit vs. single digit.
        check(1_u64, 1_u64);
        check(1_u64, 4_u64);

        // Single digit vs. multiple digits.
        check(2_u64, 42_u64);
        check(4_u64, 42_u64);
        check(5_u64, 42_u64);
        check(2_u64, 40_u64);
        check(4_u64, 40_u64);
        check(5_u64, 40_u64);

        // 2 digits vs. 2 digits.

        // Left-most digit is greater.
        check(42_u64, 24_u64);
        check(42_u64, 20_u64);

        // Left-most digit is equal.
        check(42_u64, 42_u64);
        check(42_u64, 40_u64);
        check(42_u64, 41_u64);
        check(42_u64, 43_u64);

        // Left-most digit is less.
        check(42_u64, 52_u64);
        check(42_u64, 50_u64);

        // 2 digits vs more-than-2 digits.

        // Left most digit is greater.
        check(42_u64, 200_u64);
        check(42_u64, 240_u64);
        check(42_u64, 241_u64);
        check(42_u64, 2410_u64);

        // Left most digit is equal.
        check(42_u64, 410_u64);
        check(42_u64, 411_u64);
        check(42_u64, 4100_u64);
        check(42_u64, 4110_u64);
        check(42_u64, 4111_u64);
        check(42_u64, 420_u64);
        check(42_u64, 421_u64);
        check(42_u64, 4211_u64);
        check(42_u64, 4200_u64);
        check(42_u64, 4210_u64);
        check(42_u64, 430_u64);
        check(42_u64, 431_u64);
        check(42_u64, 4300_u64);
        check(42_u64, 4310_u64);
        check(42_u64, 4311_u64);

        // Left-most digit is greater.
        check(42_u64, 500_u64);
        check(42_u64, 540_u64);
        check(42_u64, 542_u64);
        check(42_u64, 5420_u64);

        // Works with max values.
        check(u8::MAX, 1);
        check(u8::MAX, u8::MAX - 1);
        check(u16::MAX, 1);
        check(u16::MAX, u16::MAX - 1);
        check(u32::MAX, 1);
        check(u32::MAX, u32::MAX - 1);
        check(u64::MAX, 1);
        check(u64::MAX, u64::MAX - 1);
        check(usize::MAX, 1);
        check(usize::MAX, usize::MAX - 1);
        check(u128::MAX, 1);
        check(u128::MAX, u128::MAX - 1);
    }
}
