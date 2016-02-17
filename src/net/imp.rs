use sync::Arc;

use vec::Vec;
use vec_deque::VecDeque;

use sync::spin::{InterruptSpinLock, RwLock};

use thread::WaitQueue;

use net::Interface;

use hal::net::discover;

use net::Packet;
use net::defs::Device;


const MAX_QUEUE_SIZE: usize = 512;

pub struct StackImpl {
    /// Interfaces registered
    interfaces: Vec<Arc<RwLock<Interface>>>,
    /// Contains packets to be processed
    rx_queue: InterruptSpinLock<VecDeque<Packet>>,
    /// Used to wait for packet to arrive in the rx_queue
    rx_wait: WaitQueue,
}

// rx_queue is protected by a spin lock
// rx_wait is Sync
unsafe impl Sync for StackImpl {}

impl StackImpl {
    pub fn network_thread(impl_ptr: *mut StackImpl) {
        let imp: &mut StackImpl = unsafe { &mut *impl_ptr };

        loop {
            let pkt_opt = imp.rx_queue.lock().pop_front();

            match pkt_opt {
                None => {
                    imp.refresh_interfaces();
                    // No packet to process => wait for one to come
                    wait_event!(imp.rx_wait, !imp.rx_queue.lock().is_empty());
                    imp.refresh_interfaces();
                }
                Some(pkt) => {
                    // Treat the packet
                    println!("New incoming packet!");
                }
            }
        }
    }

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

        StackImpl {
            interfaces: intfs,
            rx_queue: InterruptSpinLock::new(VecDeque::with_capacity(MAX_QUEUE_SIZE)),
            rx_wait: WaitQueue::new(),
        }
    }

    pub fn interfaces(&self) -> &[Arc<RwLock<Interface>>] {
        &self.interfaces[..]
    }

    /// Call refresh on every registered interface
    fn refresh_interfaces(&mut self) {
        for intf in &mut self.interfaces {
            intf.write().refresh();
        }
    }

    pub fn enqueue_rx_packet(&mut self, packet: Packet) -> bool {
        let mut locked_rx_queue = self.rx_queue.lock();

        if locked_rx_queue.len() == MAX_QUEUE_SIZE {
            // Queue is full, we don't want to cause a reallocation in
            // interruption context. So we don't enqueue the packet
            return false;
        }

        // Enqueue the packet
        locked_rx_queue.push_back(packet);

        // Wake up network thread
        self.rx_wait.unblock();

        true
    }
}
