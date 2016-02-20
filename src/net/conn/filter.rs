//! Definitions of types and traits related to packet filtering
//!
//! Two different parameters class are going to be mentioned in this module:
//!
//! - Generic parameter: ether type, protocol id, ...
//! - Specific parameter: hardware address, ip address, ...

use net::Packet;

/// Sanitize an incoming packet.
///
/// Packets that arrive from the network need to be checked by every used
/// protocols for correctness (i.e. checksum, ...).
///
/// Some properties or extra information might need to be added to the packet
/// as well.
///
/// This trait let protocols do this.
pub trait PacketSanitizer {
    /// This method is called when a filter needs to sanitize a packet.
    ///
    /// If this method returns an `Err` the packet will be dropped.
    fn sanitize(pkt: &mut Packet) -> Result<(), ()>;
}
