use std::borrow::Borrow;
use std::convert::Infallible;
use std::fmt::Display;
use std::ops::Deref;
use std::pin::Pin;

use super::FmtEq;

/// A marker trait for types whose ordering is the same as ordering between its `Display`
/// representation.
///
/// When `T` implements `FmtOrd`, the following property must be upheld for all `a: T` and `b: T`:
///
/// ```
/// # let (a, b) = ("", "");
/// assert_eq!(a.cmp(&b), (*format!("{}", a)).cmp(&format!("{}", b)));
/// ```
///
/// In other words (assuming that no ill-defined specialization is involved):
///
/// ```text
/// a ⋛ b <-> a.to_string() ⋛ b.to_string()
/// ```
///
/// ## Colloraries
///
/// From `str: Ord` and the above property, it follows that `T` satisfies [`Ord`](std::cmp::Ord)
/// trait's contract.
///
/// ## Examples
///
/// Integer primitives do not satisfy the property.
///
/// ```
/// assert!(42 < 240);
/// // but...
/// assert!(42.to_string() > 240.to_string());
/// ```
pub trait FmtOrd: Display + Ord + FmtEq {}

// Blanket impls for `#[fundamental]` pointer types.
impl<T: FmtOrd + ?Sized> FmtOrd for &T {}
impl<T: FmtOrd + ?Sized> FmtOrd for &mut T {}
impl<P: Borrow<P::Target> + Deref + Display> FmtOrd for Pin<P> where P::Target: FmtOrd {}

impl FmtOrd for str {}
// Both `false < true` and `"false" < "true"` hold coincidentally.
impl FmtOrd for bool {}

impl FmtOrd for Infallible {}

// `alloc` types.
#[cfg(feature = "alloc")]
impl<T: FmtOrd + ?Sized> FmtOrd for alloc::boxed::Box<T> {}
#[cfg(feature = "alloc")]
impl<T: FmtOrd + ?Sized> FmtOrd for alloc::rc::Rc<T> {}
#[cfg(feature = "alloc")]
impl<T: FmtOrd + ?Sized> FmtOrd for alloc::sync::Arc<T> {}
#[cfg(feature = "alloc")]
impl<T: FmtOrd + alloc::borrow::ToOwned + ?Sized> FmtOrd for alloc::borrow::Cow<'_, T> where
    T::Owned: Display
{
}
#[cfg(feature = "alloc")]
impl FmtOrd for alloc::string::String {}
