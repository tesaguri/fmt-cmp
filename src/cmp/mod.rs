//! Stringy comparison utility.

mod generic;
#[cfg(fmt_cmp_semver_exempt)]
mod spec;

use std::cmp::Ordering;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::mem;

use super::{FmtEq, FmtOrd};

#[cfg(not(fmt_cmp_semver_exempt))]
use self::generic as imp;
#[cfg(fmt_cmp_semver_exempt)]
use self::spec as imp;

/// A wrapper type that compares the inner value in its `Display` representation.
///
/// This implements [`Eq`][std::cmp::Eq], [`Ord`][std::cmp::Ord] and [`Hash`][std::hash::Hash]
/// traits with [`eq`], [`cmp`] and [`hash`] functions.
///
/// ## Example
///
/// Wrapping `!FmtOrd` types:
///
/// ```
/// assert_eq!(fmt_cmp::Cmp(f64::NAN), fmt_cmp::Cmp(f64::NAN));
/// assert!(fmt_cmp::Cmp(42) > fmt_cmp::Cmp(240));
/// ```
///
/// Sorting integers _lexicographically_:
///
#[cfg_attr(feature = "alloc", doc = " ```")]
#[cfg_attr(not(feature = "alloc"), doc = " ```ignore")]
/// # extern crate alloc as std;
/// #
/// use std::collections::BTreeSet;
///
/// let mut values: BTreeSet<fmt_cmp::Cmp<u32>> = (1..=10).map(fmt_cmp::Cmp).collect();
/// assert!(values
///    .into_iter()
///    .map(|cmp| cmp.0)
///    .eq([1, 10, 2, 3, 4, 5, 6, 7, 8, 9]));
/// ```
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Cmp<T: ?Sized = dyn Display>(pub T);

impl<T: Display + ?Sized> Cmp<T> {
    /// Wraps a reference of type `T` as a reference of `Cmp<T>`.
    #[must_use]
    pub fn from_ref(value: &T) -> &Self {
        fn inner<'a, T: ?Sized>(value: &'a T) -> &'a Cmp<T> {
            // Safety:
            // - The lifetime annotations ensure that the output does not outlive the input.
            // - The `#[repr(transparent)]` attribute ensures that `Cmp<T>` has the same layout as
            //   `T`.
            unsafe { mem::transmute::<&'a T, &'a Cmp<T>>(value) }
        }
        inner(value)
    }

    /// Converts a `Box<T>` into `Box<Cmp<T>>`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn from_boxed(boxed: alloc::boxed::Box<T>) -> alloc::boxed::Box<Self> {
        let leaked: &mut Cmp<T> = Cmp::from_mut(alloc::boxed::Box::leak(boxed));
        // Safety:
        // - The `#[repr(transparent)]` attribute ensures that `Cmp<T>` has the same layout as `T`.
        // - `leaked` points at a block of memory currently allocated via the `Global` allocator.
        unsafe { alloc::boxed::Box::<Cmp<T>>::from_raw(leaked) }
    }

    /// Converts a `Box<Cmp<T>>` into a `Box<T>`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn into_boxed_inner(self: alloc::boxed::Box<Self>) -> alloc::boxed::Box<T> {
        let leaked: &mut T = &mut alloc::boxed::Box::leak(self).0;
        // Safety:
        // - The `#[repr(transparent)]` attribute ensures that `Cmp<T>` has the same layout as `T`.
        // - `leaked` points at a block of memory currently allocated via the `Global` allocator.
        unsafe { alloc::boxed::Box::<T>::from_raw(leaked) }
    }

    #[cfg(feature = "alloc")]
    fn from_mut(value: &mut T) -> &mut Self {
        fn inner<'a, T: ?Sized>(value: &'a mut T) -> &'a mut Cmp<T> {
            // Safety:
            // - The lifetime annotations ensure that the output does not outlive the input.
            // - The `#[repr(transparent)]` attribute ensures that `Cmp<T>` has the same layout as
            //   `T`.
            unsafe { mem::transmute::<&'a mut T, &'a mut Cmp<T>>(value) }
        }
        inner(value)
    }
}

impl<T> AsRef<T> for Cmp<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: Default + Display> Default for Cmp<T> {
    fn default() -> Self {
        Cmp(T::default())
    }
}

// `AsRef<Cmp<T>> for T` cannot be implemented due to conflict with
// `AsRef<U> for &T where T: AsRef<U>`.
impl<'a, T: Display + ?Sized> From<&'a T> for &'a Cmp<T> {
    fn from(t: &T) -> &Cmp<T> {
        Cmp::from_ref(t)
    }
}

#[cfg(feature = "alloc")]
impl<T: Display + ?Sized> From<alloc::boxed::Box<T>> for alloc::boxed::Box<Cmp<T>> {
    fn from(boxed: alloc::boxed::Box<T>) -> Self {
        Cmp::from_boxed(boxed)
    }
}

impl<T: Display + ?Sized> Display for Cmp<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

// We _could_ implement more general `PartialEq<U>` here, but we cannot ensure symmetricity and
// transitivity of such an impl.
// e.g. `Cmp("hello") == "hello" && "hello" == CaseInsensitiveStr("HELLO")` would not necessarily
// imply `Cmp("hello") == CaseInsensitiveStr("HELLO")`.
impl<T: Display + ?Sized, U: Display + ?Sized> PartialEq<Cmp<U>> for Cmp<T> {
    fn eq(&self, other: &Cmp<U>) -> bool {
        eq(&self.0, &other.0)
    }
}

impl<T: Display + ?Sized> Eq for Cmp<T> {}

impl<T: Display + ?Sized, U: Display + ?Sized> PartialOrd<Cmp<U>> for Cmp<T> {
    fn partial_cmp(&self, other: &Cmp<U>) -> Option<Ordering> {
        Some(cmp(&self.0, &other.0))
    }
}

impl<T: Display + ?Sized> Ord for Cmp<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        cmp(&self.0, &other.0)
    }
}

impl<T: Display + ?Sized> Hash for Cmp<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash(&self.0, state)
    }
}

impl<T: Display + ?Sized> FmtEq for Cmp<T> {}
impl<T: Display + ?Sized> FmtOrd for Cmp<T> {}

/// Tests two values for equality in their `Display` representations.
///
/// This yields the same result as `lhs.to_string() == rhs.to_string()` without heap allocation.
///
/// ## Note
///
/// This may call `Display::fmt` multiple times and if it emits different strings between the calls,
/// the resulting value is unspecified.
///
/// Also, the `Display` implementations may not return error as described by the documentation of
/// [`std::fmt`]. Doing so would result in an unspecified return value or might even cause
/// a panic in a future version.
///
/// ## Examples
///
/// Comparing floating-point numbers:
///
/// ```
/// assert!(fmt_cmp::eq(&f64::NAN, &f64::NAN)); // `"NaN" == "NaN"`
/// assert!(!fmt_cmp::eq(&0.0, &-0.0)); // `"0" != "-0"`
/// ```
///
/// Comparing values of different types:
///
/// ```
/// assert!(fmt_cmp::eq(&format_args!("{:X}", 0x2A), "2A"));
/// ```
#[must_use]
pub fn eq<T: Display + ?Sized, U: Display + ?Sized>(lhs: &T, rhs: &U) -> bool {
    imp::eq(lhs, rhs)
}

/// Compares two values in their `Display` representations.
///
/// This yields the same result as `lhs.to_string().cmp(&rhs.to_string())` without heap allocation.
///
/// ## Note
///
/// This may call `Display::fmt` multiple times and if it emits different strings between the calls,
/// the resulting `Ordering` value is unspecified.
///
/// Also, the `Display` implementations may not return error as described by the documentation of
/// [`std::fmt`]. Doing so would result in an unspecified `Ordering` value or might even cause
/// a panic in a future version.
///
/// ## Examples
///
/// Comparing digits of integers _lexicographically_:
///
/// ```
/// assert!(fmt_cmp::cmp(&42, &240).is_gt());
/// ```
///
/// Comparing `format_args!`:
///
/// ```
/// assert!(fmt_cmp::cmp(&format_args!("{:X}", 0x2A), &format_args!("{:X}", 0x9)).is_le());
/// ```
#[must_use]
pub fn cmp<T: Display + ?Sized, U: Display + ?Sized>(lhs: &T, rhs: &U) -> Ordering {
    imp::cmp(lhs, rhs)
}

/// Hashes a value with respect to its `Display` representation.
///
/// This satisfies the same property as `hashee.to_string().hash(hasher)` without heap allocation,
/// although the exact hash values are not guaranteed to match. In particular, the following variant
/// of [`Hash` trait's property][hash-and-eq] holds:
///
/// ```text
/// format!("{}", k1) == format!("{}", k2) -> hash(k1) == hash(k2)
/// ```
///
/// ## Note
///
/// The `Display` implementation may not return error as described by the documentation of
/// [`std::fmt`]. Doing so would result in an unspecified hash value or might even cause
/// a panic in a future version.
///
/// [hash-and-eq]: Hash#hash-and-eq
pub fn hash<T: Display + ?Sized, H: Hasher>(hashee: &T, hasher: &mut H) {
    imp::hash(hashee, hasher)
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "alloc"))]
    extern crate alloc;

    use alloc::string::ToString;
    use std::fmt::{Debug, Formatter};

    use super::*;

    #[test]
    fn fmt_cmp() {
        #[derive(Debug)]
        struct SplitFmt<'a>(&'a str, usize);
        impl Display for SplitFmt<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                let SplitFmt(s, n) = *self;
                let mut pos = 0;
                s.split_inclusive(|_| {
                    let ret = n == 0 || (pos != 0 && pos % n == 0);
                    pos += 1;
                    ret
                })
                .try_for_each(|s| f.write_str(s))
            }
        }

        #[track_caller]
        fn check<T: Debug + Display, U: Debug + Display>(x: T, y: U) {
            let (x_str, y_str) = (x.to_string(), y.to_string());
            let expected = x_str.cmp(&y_str);

            assert_eq!(cmp(&x, &y), expected);
            assert_eq!(cmp(&y, &x), expected.reverse(), "rev");
            assert_eq!(generic::cmp(&x, &y), expected, "generic");
            assert_eq!(generic::cmp(&y, &x), expected.reverse(), "generic,rev");

            for s in [&*x_str, &*y_str] {
                for n in 0..s.len() {
                    let split = SplitFmt(s, n);
                    assert_eq!(split.to_string(), s, "`{:?}` is broken", split);
                }
            }

            for (nx, ny) in (0..x_str.len()).flat_map(|i| (0..y_str.len()).map(move |j| (i, j))) {
                let (x, y) = (SplitFmt(&x_str, nx), SplitFmt(&y_str, ny));

                assert_eq!(cmp(&x, &y), expected, "{:?}", (nx, ny));
                assert_eq!(cmp(&y, &x), expected.reverse(), "{:?},rev", (nx, ny));
                assert_eq!(generic::cmp(&x, &y), expected, "generic,{:?}", (nx, ny));
                assert_eq!(
                    generic::cmp(&y, &x),
                    expected.reverse(),
                    "generic,{:?},rev",
                    (nx, ny)
                );
            }
        }

        // Empty inputs.
        check("", "");

        // Empty and non-empty inputs.
        check("", 42);

        // `lhs == rhs && lhs.to_string() == rhs.to_string()`
        check("abracadabra", "abracadabra");

        // `lhs == rhs && lhs.to_string() != rhs.to_string()`
        check(0., -0.);

        // `lhs != rhs && lhs.to_string() == rhs.to_string()`
        check(f64::NAN, f64::NAN);

        // `lhs < rhs && lhs.to_string() > rhs.to_string()`
        // `lhs.to_string() > rhs.to_string() && lhs.to_string().len() < rhs.to_string().len()`
        check(42, 240);

        // `lhs > rhs && lhs.to_string() > rhs.to_string()`
        // `lhs.to_string() > rhs.to_string() && lhs.to_string().len() > rhs.to_string().len()`
        check(42, 2);

        // One is a prefix of the other.
        check("abracadabra", "abracad");

        // Have a common prefix.
        check("abracadabra", "abrabanana");
    }

    #[test]
    fn soundness() {
        let _ = &Cmp::from_ref(&1);
        #[cfg(feature = "alloc")]
        {
            let _ = Cmp::from_boxed(alloc::boxed::Box::new(1)).into_boxed_inner();
        }

        // ZST
        let _ = Cmp::from_ref(&std::fmt::Error);
        #[cfg(feature = "alloc")]
        {
            let _ = Cmp::from_boxed(alloc::boxed::Box::new(std::fmt::Error)).into_boxed_inner();
        }

        // DST
        let _ = Cmp::from_ref("hello");
        #[cfg(feature = "alloc")]
        {
            let _ = Cmp::from_boxed(alloc::string::String::from("hello").into_boxed_str())
                .into_boxed_inner();
        }

        // Trait object
        let _ = <Cmp>::from_ref(&1);
        #[cfg(feature = "alloc")]
        {
            let _ = <Cmp>::from_boxed(alloc::boxed::Box::new(1)).into_boxed_inner();
        }
    }
}
