//! Various definitions of types, constants, traint, ... related to network

use num::PrimInt;

use core::fmt::{
    Binary,
    Octal,
    LowerHex,
    UpperHex,
    Debug,
    Display,
    Formatter,
    Error
};

/// Network integer representation
///
/// This type stores an integer using network's endianness and let the user
/// manipulates it using host's endianness.
#[derive(Clone)]
#[repr(C, packed)]
pub struct Int<T: PrimInt + Clone>(T);

impl<T> Int<T> where T: PrimInt + Clone {
    /// Construct from an integer represented using network's endianness
    pub fn from_net(i: T) -> Self {
        Int(i)
    }

    /// Construct from an integer represented using host's endianness
    pub fn from_host(i: T) -> Self {
        Int(i.to_be())
    }

    /// Returns the contained integer using host's endianness
    pub fn as_host(&self) -> T {
        T::from_be(self.0.clone())
    }

    /// Returns the contained integer using network's endianness
    pub fn as_net(&self) -> T {
        self.0.clone()
    }
}

impl<T> Binary for Int<T> where T: PrimInt + Clone + Binary {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> { self.as_host().fmt(f) }
}

impl<T> Octal for Int<T> where T: PrimInt + Clone + Octal {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> { self.as_host().fmt(f) }
}

impl<T> LowerHex for Int<T> where T: PrimInt + Clone + LowerHex {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> { self.as_host().fmt(f) }
}

impl<T> UpperHex for Int<T> where T: PrimInt + Clone + UpperHex {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> { self.as_host().fmt(f) }
}

impl<T> Debug for Int<T> where T: PrimInt + Clone + Debug {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> { self.as_host().fmt(f) }
}

impl<T> Display for Int<T> where T: PrimInt + Clone + Display {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> { self.as_host().fmt(f) }
}
