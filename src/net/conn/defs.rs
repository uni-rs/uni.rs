//! Definition of various types related to connexions

use net::defs::{HwAddr, IpAddr};

/// Type of the ether_type
pub type EtherType = u16;
/// Type that represent a protocol id
pub type ProtocolIdType = u8;
/// Type that represent a port
pub type PortType = u16;

/// Ethernet layer part of the rule
pub struct EthernetRule {
    pub ether_type: EtherType,
    pub hw_src: Option<HwAddr>,
}

/// Network layer part of the rule
pub struct NetworkRule {
    pub protocol_id: ProtocolIdType,
    pub ip_src: Option<IpAddr>,
}

/// Transport layer part of the rule
pub struct TransportRule {
    pub port: PortType,
}

/// Represent a rule that a packet must match to be enqueued in a connexion
pub struct Rule {
    pub eth_rule: Option<EthernetRule>,
    pub net_rule: Option<NetworkRule>,
    pub tspt_rule: Option<TransportRule>,
}
