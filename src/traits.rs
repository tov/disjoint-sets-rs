use std::fmt::Debug;

/// A type that can be used as a union-find element.
///
/// It must be safely convertible to and from `usize`.
///
/// The two methods must be well-behaved partial inverses as follows:
///
/// -  For all `n: usize`, if `Self::from_usize(n)` = `Some(t)` then
///    `t.to_usize()` = `n`.
/// -  For all `t: Self`, if `t.to_usize()` = `n` then
///    `Self::from_usize(n)` = `Some(t)`.
/// -  For all `n: usize`, if `Self::from_usize(n)` = `None` then for all
///    `m: usize` such that `m > n`, `Self::from_usize(m)` = `None`.
///
/// In other words, `ElementType` sets up a bijection between the first
/// *k* `usize` values and some *k* values of the `Self` type.
pub trait ElementType : Copy + Debug + Eq {
    /// Converts from `usize` to the element type.
    ///
    /// Returns `None` if the argument won’t fit in `Self`.
    fn from_usize(n: usize) -> Option<Self>;

    /// Converts from the element type to `usize`.
    fn to_usize(self) -> usize;
}

impl ElementType for usize {
    #[inline]
    fn from_usize(n: usize) -> Option<usize> { Some(n) }
    #[inline]
    fn to_usize(self) -> usize { self }
}

macro_rules! element_type_impl {
    ($type_:ident) => {
        impl ElementType for $type_ {
            #[inline]
            fn from_usize(u: usize) -> Option<Self> {
                let result = u as $type_;
                if result as usize == u { Some(result) } else { None }
            }

            #[inline]
            fn to_usize(self) -> usize {
                self as usize
            }
        }
    }
}

element_type_impl!(u8);
element_type_impl!(u16);
element_type_impl!(u32);
