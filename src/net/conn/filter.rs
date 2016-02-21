//! Definitions of types and traits related to packet filtering
//!
//! Two different parameters class are going to be mentioned in this module:
//!
//! - Generic parameter: ether type, protocol id, ...
//! - Specific parameter: hardware address, ip address, ...

use sync::Arc;

use net::Packet;

use net::defs::Rule;

use net::conn::MultiConn;

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

/// Trait implemented by generic filters (i.e. filters based on generic
/// parameters)
///
/// Generic filters allow protocols to route packets to the appropriate
/// connexion based on a generic parameter.
///
/// It may drop the packet if no matching connexion exists or if the packet is
/// invalid (failed checksum, ...).
pub trait GenericFilterTrait {
    /// Insert a new multi connexion to the filter based on a rule.
    fn insert_multi(&mut self, conn: Arc<MultiConn>,
                    rule: &Rule) -> Result<(), ()>;

    /// Filter and route an incoming packet to a connexion (uni or multi).
    fn rx(&self, pkt: Packet);

    /// Filter and route an incoming packet to a multi connexion
    fn rx_multi(&self, pkt: Packet, rule: Rule);
}

/// Trait implemented by specific filters (i.e. filters based on specific
/// parameters)
///
/// Specific filters allow protocols to route packets to the appropriate
/// connexion based on a specific parameter
///
/// It may drop the packet if no matching connexion exists.
pub trait SpecificFilterTrait<T> {
    /// Create a new specific filter.
    ///
    /// The generic parameter taken as parameter is used to associate a
    /// specific filter with a generic one.
    fn new(generic_param: T) -> Self;

    /// Insert a new multi connexion to the filter based on a rule.
    fn insert_multi(&mut self, conn: Arc<MultiConn>,
                    rule: &Rule) -> Result<(), ()>;

    /// Filter and route an incoming packet to a connexion (uni or multi).
    fn rx(&self, pkt: Packet);

    /// Filter and route an incoming packet to a multi connexion
    fn rx_multi(&self, pkt: Packet, rule: Rule);
}
