//! Numeric traits and functions for the built-in numeric types.

pub use core::num::*;

/// Primitive operations for built-in integer types
pub trait PrimInt: Sized {
    /// Converts an integer from big endian to the target's endianness
    fn from_be(x: Self) -> Self;

    /// Converts an integer from little endian to the target's endianness
    fn from_le(x: Self) -> Self;

    /// Converts `self` to big endian from the target's endianness
    fn to_be(self) -> Self;

    /// Converts `self` to little endian from the target's endianness
    fn to_le(self) -> Self;
}

macro_rules! prim_int {
    ($($t:ident)*) => ($(
        impl PrimInt for $t {
            #[inline]
            fn from_be(x: Self) -> Self { $t::from_be(x) }
            #[inline]
            fn from_le(x: Self) -> Self { $t::from_le(x) }
            #[inline]
            fn to_be(self) -> Self { self.to_be() }
            #[inline]
            fn to_le(self) -> Self { self.to_le() }
        }
    )*)
}

prim_int!(i8 i16 i32 i64 isize u8 u16 u32 u64 usize);
