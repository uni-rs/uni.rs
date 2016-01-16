use net::defs::Ipv4Addr;

// XXX: Should this be in net::defs ?
/// IPv4 configuration of an interface
pub struct V4Configuration {
    /// Main IPv4 address
    pub ipv4: Ipv4Addr,
    /// Subnet mask
    pub ipv4_mask: Ipv4Addr,
    /// Gateway IPv4 address
    pub ipv4_gateway: Ipv4Addr,
}
