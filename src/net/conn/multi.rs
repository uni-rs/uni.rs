use vec_deque::VecDeque;

use sync::spin::SpinLock;

use thread::WaitQueue;

use net::{InterfaceWeak, Packet, PacketBuilder};

use net::defs::Rule;

/// Connexion that can receive packets from multiple endpoints.
///
/// This is just a placeholder object. In order for it to receive packets it
/// must be added to a `Manager`.
pub struct MultiConn {
    queue: SpinLock<VecDeque<(Packet, Rule)>>,
    wait: WaitQueue,
    parent: InterfaceWeak,
}

unsafe impl Sync for MultiConn {}

impl MultiConn {
    /// Create a new multi connexion object.
    ///
    /// This should not be used directly. Instead, to create a new multi
    /// connexion, use `Interface::create_multi()`.
    pub fn new(parent: InterfaceWeak) -> Self {
        MultiConn {
            queue: SpinLock::new(VecDeque::new()),
            wait: WaitQueue::new(),
            parent: parent,
        }
    }

    /// Pop a packet from the connexion.
    ///
    /// It returns a tuple that contains the packet and a rule that contains
    /// information to reply to the sender.
    ///
    /// Note that if no packets are available, this function will block until
    /// one is received.
    pub fn pop_packet(&self) -> (Packet, Rule) {
        loop {
            let res = self.queue.lock().pop_front();

            match res {
                None => wait_event!(self.wait, !self.queue.lock().is_empty()),
                Some(res) => return res,
            }
        }
    }

    /// Insert a packet inside the connexion
    ///
    /// `pkt` is the received packet
    ///
    /// `rule` is information about the endpoint that sent the packet
    pub fn rx(&self, pkt: Packet, rule: Rule) {
        self.queue.lock().push_back((pkt, rule));
        self.wait.unblock();
    }

    /// Send data through the connexion.
    ///
    /// Since a multi connexion does not target a specific endpoint you have
    /// to pass a `rule` to tell the connexion the endpoint the packet has to
    /// be sent to.
    pub fn tx(&self, data: &[u8], rule: &Rule) -> Result<(), ()> {
        let mut builder = try!(PacketBuilder::new());

        try!(builder.write(data));

        self.tx_packet(builder, rule)
    }

    /// Send a packet through the connexion.
    ///
    /// It might ease the pain of some data sender to use directly a
    /// `PacketBuilder` and send this directly without using a slice.
    ///
    /// Since a multi connexion does not target a specific endpoint you have
    /// to pass a `rule` to tell the connexion the endpoint the packet has to
    /// be sent to.
    pub fn tx_packet(&self, builder: PacketBuilder,
                     rule: &Rule) -> Result<(), ()> {
        let intf = try!(self.parent.upgrade().ok_or(()));

        intf.tx_packet(builder, rule)
    }
}

impl Drop for MultiConn {
    fn drop(&mut self) {
        // XXX: The way multi connexion should go down is not defined yet.
        // This will eventually trigger when it's time to seriously think
        // about it
        unimplemented!();
    }
}
