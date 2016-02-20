//! Definitions of types and traits related to packet filtering
//!
//! Two different parameters class are going to be mentioned in this module:
//!
//! - Generic parameter: ether type, protocol id, ...
//! - Specific parameter: hardware address, ip address, ...

use net::Packet;

use net::defs::Rule;

/// Extract generic/specific parameters of type T from a packet or a rule.
///
/// This is used by filters to extract the generic/specific parameter and use
/// it to route the packet to the appropriate connexion (if such connexion
/// exists).
pub trait Extractor<T> {
    /// Extract the parameter from a rule.
    ///
    /// This will return `None` if the method fails to extract the parameter.
    fn from_rule(rule: &Rule) -> Option<T>;

    /// Extract the parameter from a Packet.
    ///
    /// This will return `None` if the method fails to extract the parameter.
    fn from_packet(pkt: &Packet) -> Option<T>;
}

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
