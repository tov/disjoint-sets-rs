use std::fmt::Debug;

/// A type that can be used as a union-find element.
///
/// It must be safely convertible to and from `usize`.
///
/// The two methods must be well-behaved partial inverses as follows:
///
/// -  For all `f: usize`, if `f::from_usize()` = `Some(t)` then
///    `t::to_usize()` = `f`.
/// -  For all `f: Self`, if `f::to_usize()` = `t` then
///    `t::from_usize()` = `Some(f)`.
/// -  For all `f: usize`, if `f::from_usize()` = `None` then for all `g:
///    usize` such that `g > f`, `g::from_usize()` = `None`.
///
/// In other words, `ElementType` sets up a bijection between the first
/// *k* `usize` values and some *k~ values of the `Self` type.
pub trait ElementType : Copy + Debug + Eq {
    /// Converts from `usize` to the element type.
    ///
    /// Returns `None` if the argument won’t fit in `Self`.
    #[inline]
    fn from_usize(usize) -> Option<Self>;

    /// Converts from the element type to `usize`.
    #[inline]
    fn to_usize(self) -> usize;
}

impl ElementType for usize {
    fn from_usize(n: usize) -> Option<usize> { Some(n) }
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
