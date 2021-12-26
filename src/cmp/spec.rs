use std::cmp::Ordering;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

use super::generic;

pub fn eq<T: Display + ?Sized, U: Display + ?Sized>(lhs: &T, rhs: &U) -> bool {
    SpecEq::spec_eq(lhs, rhs)
}

pub fn cmp<T: Display + ?Sized, U: Display + ?Sized>(lhs: &T, rhs: &U) -> Ordering {
    SpecOrd::spec_cmp(lhs, rhs)
}

pub fn hash<T: Display + ?Sized, H: Hasher>(hashee: &T, hasher: &mut H) {
    SpecHash::spec_hash(hashee, hasher)
}

trait SpecEq<T: ?Sized = Self> {
    fn spec_eq(&self, other: &T) -> bool;
}

trait SpecOrd<T: ?Sized = Self> {
    fn spec_cmp(&self, other: &T) -> Ordering;
}

trait SpecHash {
    fn spec_hash<H: Hasher>(&self, state: &mut H);
}

impl<T: Display + ?Sized, U: Display + ?Sized> SpecEq<U> for T {
    default fn spec_eq(&self, other: &U) -> bool {
        generic::eq(self, other)
    }
}

impl<T: Display + ?Sized, U: Display + ?Sized> SpecOrd<U> for T {
    default fn spec_cmp(&self, other: &U) -> Ordering {
        generic::cmp(self, other)
    }
}

impl<T: Display + ?Sized> SpecHash for T {
    default fn spec_hash<H: Hasher>(&self, state: &mut H) {
        generic::hash(self, state)
    }
}

macro_rules! naive_eq {
    ($($ty:ty)*) => {$(
        impl SpecEq for $ty {
            fn spec_eq(&self, other: &Self) -> bool {
                *self == *other
            }
        }

        impl SpecEq<&$ty> for &$ty {
            fn spec_eq(&self, other: &&$ty) -> bool {
                **self == **other
            }
        }

        impl SpecHash for $ty {
            fn spec_hash<H: Hasher>(&self, state: &mut H) {
                Hash::hash(self, state)
            }
        }

        impl SpecHash for &$ty {
            fn spec_hash<H: Hasher>(&self, state: &mut H) {
                Hash::hash(*self, state)
            }
        }
    )*};
}

naive_eq! {
    u8 u16 u32 u64 usize u128
    i8 i16 i32 i64 isize i128
    bool
}

/// Generates `impl SpecOrd<U> for T` for every permutation of the input types and their references.
/// The input types must be deref-coercible to `str`.
macro_rules! str_cmp {
    ($($(#[$attr:meta])* $ty:ty;)*) => {
        str_cmp! {
            { $($(#[$attr])* $ty;)* $($(#[$attr])* &$ty;)* }
            { $($(#[$attr])* $ty;)* $($(#[$attr])* &$ty;)* }
            { $($(#[$attr])* $ty;)* $($(#[$attr])* &$ty;)* }
        }
    };
    (
        { $(#[$t_attr:meta])* $t:ty; $($rest_t:tt)* }
        { $(#[$u_attr:meta])* $u:ty; $($rest_u:tt)* }
        $orig:tt
    ) => {
        $(#[$t_attr])*
        $(#[$u_attr])*
        impl SpecEq<$u> for $t {
            fn spec_eq(&self, other: &$u) -> bool {
                let (this, other): (&str, &str) = (self, other);
                *this == *other
            }
        }

        $(#[$t_attr])*
        $(#[$u_attr])*
        impl SpecOrd<$u> for $t {
            fn spec_cmp(&self, other: &$u) -> Ordering {
                str::cmp(self, other)
            }
        }

        // Shift `$u`.
        str_cmp! {
            { $(#[$t_attr])* $t; $($rest_t)* }
            { $($rest_u)* }
            $orig
        }
    };
    (
        { $(#[$attr:meta])* $t:ty; $($rest:tt)* }
        {}
        $orig:tt
    ) => {
        $(#[$attr])*
        impl SpecHash for &$t {
            fn spec_hash<H: Hasher>(&self, state: &mut H) {
                Hash::hash(*self, state)
            }
        }

        // Shift `$t` and reset `$u`.
        str_cmp! { { $($rest)* } $orig $orig }
    };
    ({} $_u:tt $_orig:tt) => {};
}

str_cmp! {
    str;
    #[cfg(feature = "alloc")]
    alloc::string::String;
    #[cfg(feature = "alloc")]
    alloc::boxed::Box<str>;
    #[cfg(feature = "alloc")]
    alloc::borrow::Cow<'_, str>;
}

macro_rules! int_ord {
    ($($ty:ty)*) => {$(
        impl SpecOrd for $ty {
            fn spec_cmp(&self, other: &Self) -> Ordering {
                crate::cmp_dec(*self, *other)
            }
        }

        impl SpecOrd<&$ty> for &$ty {
            fn spec_cmp(&self, other: &&$ty) -> Ordering {
                crate::cmp_dec(**self, **other)
            }
        }
    )*};
}

int_ord! { u8 u16 u32 u64 usize u128 }
