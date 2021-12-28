use std::borrow::Borrow;
use std::convert::Infallible;
use std::fmt::Display;
use std::ops::Deref;
use std::pin::Pin;

/// A marker trait for types whose equivalence relation is the same as equivalence between its
/// `Display` representation.
///
/// When `T` implements `FmtEq`, the following property must be upheld for all `a: T` and `b: T`:
///
/// ```
/// # let (a, b) = ("", "");
/// assert_eq!(a == b, *format!("{}", a) == *format!("{}", b));
/// ```
///
/// In other words (assuming that no ill-defined specialization is involved):
///
/// ```text
/// a == b <-> a.to_string() == b.to_string()
/// ```
///
/// ## Corollaries
///
/// From `str: Eq` and the above property, it follows that `T` satisfies [`Eq`](std::cmp::Eq)
/// trait's contract, i.e., reflexivity, symmetricity and transitivity of `==` operator.
///
/// ## Examples
///
/// Floating-point number primitives do not satisfy the property (they are not even `Eq`):
///
/// ```
/// assert_eq!(0.0, -0.0);
/// assert_ne!(0.0.to_string(), (-0.0).to_string());
///
/// assert_ne!(f64::NAN, f64::NAN);
/// assert_eq!(f64::NAN.to_string(), f64::NAN.to_string());
/// ```
///
/// Wrapping any `Display` type with [`fmt_cmp::Cmp`](crate::Cmp) makes it `FmtEq`:
///
/// ```
/// assert_ne!(fmt_cmp::Cmp(0.0), fmt_cmp::Cmp(-0.0));
/// assert_eq!(fmt_cmp::Cmp(f64::NAN), fmt_cmp::Cmp(f64::NAN));
/// ```
pub trait FmtEq: Display + Eq {}

// Blanket impls for `#[fundamental]` pointer types.
impl<T: FmtEq + ?Sized> FmtEq for &T {}
impl<T: FmtEq + ?Sized> FmtEq for &mut T {}
// `Pin<P>` implements `Display` via `<P as Display>` and `PartialEq` with
// `<P::Target as PartialEq>`. The `P: Borrow<P::Target>` bound should ensure that `<P as Display>`
// behaves identically with `<P::Target as Display>`.
// This implementation covers `Pin<&T>`, `Pin<&mut T>` and `Pin<Box<T>>`.
impl<P: Borrow<P::Target> + Deref + Display> FmtEq for Pin<P> where P::Target: FmtEq {}

impl FmtEq for str {}
impl FmtEq for bool {}

impl FmtEq for Infallible {}

// `alloc` types.
#[cfg(feature = "alloc")]
impl<T: FmtEq + ?Sized> FmtEq for alloc::boxed::Box<T> {}
#[cfg(feature = "alloc")]
impl<T: FmtEq + ?Sized> FmtEq for alloc::rc::Rc<T> {}
#[cfg(feature = "alloc")]
impl<T: FmtEq + ?Sized> FmtEq for alloc::sync::Arc<T> {}
// We can assume that `Display` and `PartialEq` behave consistently between `T` and `T::Owned`
// because of `Borrow<T>` trait's contract, which is implemented by `T::Owned`.
#[cfg(feature = "alloc")]
impl<T: FmtEq + alloc::borrow::ToOwned + ?Sized> FmtEq for alloc::borrow::Cow<'_, T> where
    T::Owned: Display
{
}
#[cfg(feature = "alloc")]
impl FmtEq for alloc::string::String {}

// `!FmtOrd` types:

// Integral primitives.
impl FmtEq for u8 {}
impl FmtEq for u16 {}
impl FmtEq for u32 {}
impl FmtEq for u64 {}
impl FmtEq for u128 {}
impl FmtEq for usize {}
impl FmtEq for i8 {}
impl FmtEq for i16 {}
impl FmtEq for i32 {}
impl FmtEq for i64 {}
impl FmtEq for i128 {}
impl FmtEq for isize {}

// TODO: Does `char` satisfy the trait contract?
