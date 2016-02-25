//! Useful callbacks used by ethernet's specific filter

use boxed::Box;

use net::Packet;

use net::defs::{Rule, EthernetRule, EtherType, ETHERTYPE_IPV4, ETHERTYPE_IPV6};

use net::conn::filter::{SpecificCallbacks, GenericFilterTrait};

use net::ipv4::Ipv4GenericFilter;

use super::defs::Header;

/// Defines specific callbacks for ethernet protocol
pub struct EthernetCallbacks;

impl SpecificCallbacks<EtherType> for EthernetCallbacks {
    /// Create a network filter based on the ether type
    fn filter_from_generic_parameter(ether_type: EtherType) -> Option<Box<GenericFilterTrait>> {
        match ether_type {
            ETHERTYPE_IPV4 => Some(Box::new(Ipv4GenericFilter::new())),
            ETHERTYPE_IPV6 => unimplemented!(),
            _ => None,
        }
    }

    #[inline]
    /// Does the rule has a network rule component
    fn has_upper_filter(rule: &Rule) -> bool {
        rule.net_rule.is_some()
    }

    /// Set ethernet part of the rule with information gathered from the packet
    fn set_layer_rule(rule: &mut Rule, pkt: &Packet) {
        let hdr = pkt.link_header::<Header>().unwrap();

        rule.eth_rule = Some(EthernetRule {
            ether_type: hdr.ether_type.as_host(),
            hw_in: Some(hdr.src.clone()),
        });
    }
}
