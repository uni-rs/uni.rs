//! Various definitions of types, constants, traint, ... related to network

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

use num::PrimInt;

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

const COUNT_HWADDR_BYTES: usize = 6;

#[repr(C, packed)]
#[derive(Copy, Clone, PartialEq)]
/// An Ipv4 address
pub struct Ipv4Addr {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
}

impl Ipv4Addr {
    /// Creates a new IPv4 address
    ///
    /// The result will represent the IP address `a`.`b`.`c`.`d`
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Ipv4Addr {
            a: a,
            b: b,
            c: c,
            d: d,
        }
    }

}

impl Display for Ipv4Addr {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_fmt(format_args!("{}.{}.{}.{}", self.a, self.b, self.c,
                                 self.d))
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
/// A MAC address
pub struct HwAddr {
    bytes: [u8; COUNT_HWADDR_BYTES],
}

impl HwAddr {
    /// Create an empty hardware address (i.e., 00:00:00:00:00:00)
    pub fn empty() -> Self {
        HwAddr {
            bytes: [0; COUNT_HWADDR_BYTES],
        }
    }

    /// Create an hardware address from bytes.
    ///
    /// This method is unsafe because the slice *MUST* contain at least 6
    /// elements.
    pub unsafe fn from_bytes(bytes: &[u8]) -> Self {
        let mut ret = Self::empty();

        ret.bytes[0] = bytes[0];
        ret.bytes[1] = bytes[1];
        ret.bytes[2] = bytes[2];
        ret.bytes[3] = bytes[3];
        ret.bytes[4] = bytes[4];
        ret.bytes[5] = bytes[5];

        ret
    }

    #[inline]
    /// Returns the internal representation of an hardware address
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..]
    }
}

impl PartialEq for HwAddr {
    fn eq(&self, rhs: &Self) -> bool {
        self.bytes[0] == rhs.bytes[0] &&
        self.bytes[1] == rhs.bytes[1] &&
        self.bytes[2] == rhs.bytes[2] &&
        self.bytes[3] == rhs.bytes[3] &&
        self.bytes[4] == rhs.bytes[4] &&
        self.bytes[5] == rhs.bytes[5]
    }
}

impl Display for HwAddr {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_fmt(format_args!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                                 self.bytes[0], self.bytes[1], self.bytes[2],
                                 self.bytes[3], self.bytes[4], self.bytes[5]))
    }
}
