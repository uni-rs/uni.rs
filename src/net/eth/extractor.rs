//! Implementation of Ethernet related Extractors

use net::Packet;

use net::defs::{Rule, EtherType};

use net::conn::filter::Extractor;

use super::defs::Header;

/// Type responsible for ether type extraction
pub struct EtherTypeExtractor;

impl Extractor<EtherType> for EtherTypeExtractor {
    /// Extract the ether type from a rule
    fn from_rule(rule: &Rule) -> Option<EtherType> {
        rule.eth_rule.as_ref().map(|eth_rule| eth_rule.ether_type)
    }

    /// Extract the ether type from a packet
    fn from_packet(pkt: &Packet) -> Option<EtherType> {
        pkt.link_header::<Header>().map(|hdr| hdr.ether_type.as_host())
    }
}
