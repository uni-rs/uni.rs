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

use core::str::FromStr;

use vec::Vec;

use num::PrimInt;

use net::Packet;

/// Type of the ether_type
pub type EtherType = u16;
/// Type that represent a protocol id
pub type ProtocolIdType = u8;
/// Type that represent a port
pub type PortType = u16;

#[derive(Clone)]
/// Ethernet layer part of the rule
pub struct EthernetRule {
    pub ether_type: EtherType,
    pub hw_in: Option<HwAddr>,
}

#[derive(Clone)]
/// Network layer part of the rule
pub struct NetworkRule {
    pub protocol_id: ProtocolIdType,
    pub ip_in: Option<IpAddr>,
}

#[derive(Clone)]
/// Transport layer part of the rule
pub struct TransportRule {
    pub port: PortType,
}

#[derive(Clone)]
/// Represent a rule that a packet must match to be enqueued in a connexion
pub struct Rule {
    pub eth_rule: Option<EthernetRule>,
    pub net_rule: Option<NetworkRule>,
    pub tspt_rule: Option<TransportRule>,
}

/// Trait implemented by hardware interfaces
pub trait Device {
    /// Periodically called by the network thread to let the interface
    /// refresh its rx/tx buffers, ...
    fn refresh(&mut self);

    /// Transmit a packet via the interface
    fn tx_packet(&mut self, pkt: Packet);
}

/// Network integer representation
///
/// This type stores an integer using network's endianness and let the user
/// manipulates it using host's endianness.
#[repr(C, packed)]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
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

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
/// An IP address, either V4 or V6
pub enum IpAddr {
    /// An IPv4 address
    V4(Ipv4Addr),
    /// An IPv6 address
    V6(Ipv6Addr)
}

#[repr(C, packed)]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
/// An IPv4 address
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
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
/// An IPv6 address
pub struct Ipv6Addr {
    a: Int<u16>,
    b: Int<u16>,
    c: Int<u16>,
    d: Int<u16>,
    e: Int<u16>,
    f: Int<u16>,
    g: Int<u16>,
    h: Int<u16>,
}

impl Ipv6Addr {
    /// Creates a new IPv6 address
    ///
    /// The result will represent the IP address `a`:`b`:`c`:`d`:`e`:`f`:`g`:`h`
    pub fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16,
               h: u16) -> Ipv6Addr {
        Ipv6Addr {
            a: Int::from_host(a),
            b: Int::from_host(b),
            c: Int::from_host(c),
            d: Int::from_host(d),
            e: Int::from_host(e),
            f: Int::from_host(f),
            g: Int::from_host(g),
            h: Int::from_host(h),
        }
    }
}

impl Display for Ipv6Addr {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_fmt(format_args!("{}:{}:{}:{}:{}:{}:{}:{}", self.a, self.b,
                                 self.c, self.d, self.e, self.f, self.g,
                                 self.h))
    }
}

#[repr(C, packed)]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
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

    /// Create a broadcast hardware address (i.e., FF:FF:FF:FF:FF:FF)
    pub fn broadcast() -> Self {
        HwAddr {
            bytes: [0xFF; COUNT_HWADDR_BYTES],
        }
    }

    /// Is the current hardware address a broadcast
    /// address (i.e., FF:FF:FF:FF:FF:FF)
    pub fn is_broadcast(&self) -> bool {
        *self == Self::broadcast()
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

impl FromStr for HwAddr {
    type Err = ();

    /// Convert a string with format XX:XX:XX:XX:XX:XX to an hardware address
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split(':').collect();

        if split.len() != COUNT_HWADDR_BYTES {
            return Err(())
        }

        let mut ret = Self::empty();

        for (i, b) in split.iter().enumerate() {
            let bytes = b.as_bytes();

            if bytes.len() != 2 {
                return Err(())
            }

            let d1 = try!(b.char_at(0).to_digit(16).ok_or(())) * 16;
            let d2 = try!(b.char_at(1).to_digit(16).ok_or(()));

            ret.bytes[i] = d1 as u8 + d2 as u8;
        }

        Ok(ret)
    }
}

impl Display for HwAddr {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_fmt(format_args!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                                 self.bytes[0], self.bytes[1], self.bytes[2],
                                 self.bytes[3], self.bytes[4], self.bytes[5]))
    }
}
