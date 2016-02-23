use net::defs::{HwAddr, Int as NetInt};

#[repr(C, packed)]
/// Ethernet header
pub struct Header {
    pub dest: HwAddr,
    pub src: HwAddr,
    pub ether_type: NetInt<u16>,
}
