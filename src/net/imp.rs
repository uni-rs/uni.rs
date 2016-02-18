use sync::Arc;

use vec::Vec;
use vec_deque::VecDeque;

use sync::spin::InterruptSpinLock;

use thread::WaitQueue;

use net::Interface;

use hal::net::discover;

use net::Packet;
use net::defs::Device;


const MAX_QUEUE_SIZE: usize = 512;

#[derive(Clone)]
/// A network stack
///
/// This object represent a shareable network stack. This object stack
/// internally uses an Arc so it can be safely shared.
pub struct Instance(Arc<InstanceRaw>);

struct InstanceRaw {
    /// Interfaces registered
    interfaces: Vec<Interface>,
    /// Contains packets to be processed
    rx_queue: InterruptSpinLock<VecDeque<Packet>>,
    /// Used to wait for packet to arrive in the rx_queue
    rx_wait: WaitQueue,
}

// rx_queue is protected by a spin lock
// rx_wait is Sync
unsafe impl Sync for InstanceRaw {}

impl Instance {
    /// Network thread linked to an instance
    ///
    /// This function is used to process packet that are received within a
    /// network stack instance
    pub fn network_thread(instance: Instance) {
        loop {
            let pkt_opt = instance.0.rx_queue.lock().pop_front();

            match pkt_opt {
                None => {
                    instance.refresh_interfaces();
                    // No packet to process => wait for one to come
                    wait_event!(instance.0.rx_wait,
                                !instance.0.rx_queue.lock().is_empty());
                    instance.refresh_interfaces();
                }
                Some(pkt) => {
                    // Treat the packet
                    println!("New incoming packet!");
                }
            }
        }
    }

    /// Create a new network stack
    ///
    /// TODO: This cannot really be used more than once for now.
    pub fn new() -> Self {
        let intfs = discover();

        if intfs.is_empty() {
            println!("Warning: Uni.rs is built with network capabilities but no interface found");
        } else {
            println!("{} interface(s) discovered:", intfs.len());

            for i in &intfs {
                println!("  - {} ({})", i.read().name_ref(),
                         i.read().hw_addr_ref());
            }
        }

        let inner = Arc::new(InstanceRaw {
            interfaces: intfs,
            rx_queue: InterruptSpinLock::new(VecDeque::with_capacity(MAX_QUEUE_SIZE)),
            rx_wait: WaitQueue::new(),
        });

        Instance(inner)
    }

    /// Get the list of registered interfaces within network stack
    pub fn interfaces(&self) -> &[Interface] {
        &self.0.interfaces[..]
    }

    /// Call refresh on every registered interface
    fn refresh_interfaces(&self) {
        for intf in self.interfaces() {
            intf.write().refresh();
        }
    }

    /// Enqueue a packet to be received on the network stack
    ///
    /// Returns false if the receive queue is full and therefore the packet was
    /// not enqueued.
    ///
    /// Note: This is safe to be called from interrupt context. Indeed the
    /// receive queue is not resizable so no allocation will be performed by
    /// this function
    pub fn enqueue_rx_packet(&self, packet: Packet) -> bool {
        let mut locked_rx_queue = self.0.rx_queue.lock();

        if locked_rx_queue.len() == MAX_QUEUE_SIZE {
            // Queue is full, we don't want to cause a reallocation in
            // interruption context. So we don't enqueue the packet
            return false;
        }

        // Enqueue the packet
        locked_rx_queue.push_back(packet);

        // Wake up network thread
        self.0.rx_wait.unblock();

        true
    }
}
