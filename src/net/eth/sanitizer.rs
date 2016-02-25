//! Sanitize incoming packets at the ethernet layer

use core::mem;

use net::Packet;

use net::conn::filter::PacketSanitizer;

use super::defs::Header;

/// Sanitize a packet at the ethernet level
pub struct EthernetPacketSanitizer;

impl PacketSanitizer for EthernetPacketSanitizer {
    /// Determine if the packet is valid at the ethernet level and if we should
    /// accept it
    fn sanitize(pkt: &mut Packet) -> Result<(), ()> {
        // Verify the packet
        // For now we only accept packets that targets directly the interface
        // the packet was received on. The only exception is broadcast packets
        {
            // Incoming packet *MUST* have an interface set
            let intf = try!(pkt.interface().ok_or(()));

            // Get a reference over the ethernet header
            let hdr = try!(pkt.link_header::<Header>().ok_or(()));

            // For now we only accept packets that are either for us or
            // broadcast
            if *intf.read().hw_addr_ref() != hdr.dest &&
                !hdr.dest.is_broadcast() {
                return Err(())
            }
        }

        unsafe {
            // Set the size of the link layer header (i.e. size of the ethernet
            // header)
            *pkt.link_hdr_size_mut() = mem::size_of::<Header>();
        }

        // Accept packet
        Ok(())
    }
}
