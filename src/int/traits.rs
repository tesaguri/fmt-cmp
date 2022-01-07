/// A trait for integer types that can be compared with [`cmp_int`](super::cmp_int) function.
///
/// This trait is sealed and cannot be implemented outside of `fmt_cmp` crate.
pub trait Integer: private::Sealed {}

mod private {
    pub trait Sealed {
        fn copy(&self) -> Self;
        fn eq(self, other: Self) -> bool;
        fn lt(self, other: Self) -> bool;
        fn checked_log(self, base: Self) -> Option<u32>;
        fn log(self, base: u32) -> u32;
        fn checked_log10(self) -> Option<u32>;
        fn log10(self) -> u32;
        /// Calculates `self / base.pow(exp)`.
        fn invpow(self, base: u32, exp: u32) -> Self;
    }
}

macro_rules! sealed_common {
    () => {
        fn copy(&self) -> Self {
            *self
        }

        fn eq(self, other: Self) -> bool {
            self == other
        }

        fn lt(self, other: Self) -> bool {
            self < other
        }

        fn checked_log(mut self, base: Self) -> Option<u32> {
            // Well, the function isn't _checking_ anything in fact, since we are not going to call
            // it with `base == 0` and defaulting to `0` if `None` is returned.
            // It is returning `Option<_>` only for consistency with the inherent `checked_log`.
            if base <= 1 {
                assert!(base > 0);
                return Some(0);
            }
            let mut x = 0;
            while self >= base {
                self /= base;
                x += 1;
            }
            Some(x)
        }

        // `unstable_name_collisions` here are intentional. This will automatically use the inherent
        // `checked_log` if available or uses the fallback impl otherwise.
        #[allow(unstable_name_collisions)]
        fn log(self, base: u32) -> u32 {
            if let Some(x) = self.checked_log(base as _) {
                x
            } else {
                0
            }
        }

        #[allow(unstable_name_collisions)]
        fn log10(self) -> u32 {
            if let Some(x) = self.checked_log10() {
                x
            } else {
                0
            }
        }

        fn invpow(mut self, base: u32, mut exp: u32) -> Self {
            // Based on `{integer}::pow` implementation from `core`.
            // <https://doc.rust-lang.org/1.57.0/src/core/num/uint_macros.rs.html#1926-1946>
            // The variant implementation is provided to prevent overflows in cases like
            // `10_u32.pow(u128::MAX.log10() - 0_u128.log10())` without resorting to `u128::pow`.

            if exp == 0 {
                return self;
            }
            // The `exp` argument in our use case is `Self.log(base) - Self.log(base)`,
            // which would be zero if `base > Self::MAX` so the `as` conversion is lossless.
            let mut base = base as Self;

            while exp > 1 {
                if (exp & 1) == 1 {
                    self /= base;
                }
                exp /= 2;
                base = base * base;
            }

            self / base
        }
    };
}

// These specialized `log10` implementations are based on `core`'s ones.
// <https://doc.rust-lang.org/1.57.0/src/core/num/int_log10.rs.html#52-90>

impl private::Sealed for u32 {
    sealed_common!();

    #[allow(unstable_name_collisions)]
    fn checked_log10(mut self) -> Option<u32> {
        let x = if self >= 100_000 {
            self /= 100_000;
            5
        } else {
            0
        };

        // Checking that `self` would be `<= u16::MAX` now even if the argument were `u32::MAX`...
        assert!((!0_u32) / 100_000 <= (!0_u16) as u32);
        debug_assert!(self <= (!0_u16) as u32); // ... so that this holds.

        Some((self as u16).log(10) + x)
    }
}

impl private::Sealed for u64 {
    sealed_common!();

    #[allow(unstable_name_collisions)]
    fn checked_log10(mut self) -> Option<u32> {
        let x = if self >= 10_000_000_000 {
            self /= 10_000_000_000;
            10
        } else {
            0
        };
        assert!((!0_u64) / 10_000_000_000 <= (!0_u32) as u64);
        debug_assert!(self <= (!0_u32) as u64);
        Some((self as u32).log(10) + x)
    }
}

impl private::Sealed for u128 {
    sealed_common!();

    #[allow(unstable_name_collisions)]
    fn checked_log10(mut self) -> Option<u32> {
        if self >= 100_000_000_000_000_000_000_000_000_000_000 {
            self /= 100_000_000_000_000_000_000_000_000_000_000;
            assert!((!0_u128) / 100_000_000_000_000_000_000_000_000_000_000 <= (!0_u32) as u128);
            debug_assert!(self <= (!0_u32) as u128);
            return Some((self as u32).log(10) + 32);
        }
        let x = if self >= 10_000_000_000_000_000 {
            self /= 10_000_000_000_000_000;
            16
        } else {
            0
        };
        assert!(
            (100_000_000_000_000_000_000_000_000_000_000 - 1) / 10_000_000_000_000_000
                <= (!0_u64) as u128
        );
        debug_assert!(self <= (!0_u64) as u128);
        Some((self as u64).log(10) + x)
    }
}

macro_rules! generic_log10 {
    ($($ty:ty)*) => {$(
        impl private::Sealed for $ty {
            sealed_common!();

            #[allow(unstable_name_collisions)]
            fn checked_log10(self) -> Option<u32> {
                self.checked_log(10)
            }
        }
    )*};
}

generic_log10! { u8 u16 }

#[cfg(target_pointer_width = "64")]
impl private::Sealed for usize {
    sealed_common!();
    #[allow(unstable_name_collisions)]
    fn checked_log10(self) -> Option<u32> {
        (self as u64).checked_log10()
    }
}

#[cfg(target_pointer_width = "32")]
impl private::Sealed for usize {
    sealed_common!();
    #[allow(unstable_name_collisions)]
    fn checked_log10(self) -> Option<u32> {
        (self as u32).checked_log10()
    }
}

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
generic_log10! { usize }

impl Integer for u8 {}
impl Integer for u16 {}
impl Integer for u32 {}
impl Integer for u64 {}
impl Integer for u128 {}
impl Integer for usize {}
