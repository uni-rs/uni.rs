use sync::{Arc, Weak};

use vec::Vec;
use vec_deque::VecDeque;

use cell::{GlobalCell, GlobalCellRef};

use sync::spin::{InterruptSpinLock, RwLock};

use thread::{Scheduler, WaitQueue};

use net::Interface;

use hal::net::discover;

use net::Packet;
use net::defs::Device;

const MAX_QUEUE_SIZE: usize = 512;

pub static STACK: GlobalCell<StackImpl> = GlobalCell::new();

/// Uni.rs network stack
pub struct Stack;

impl Stack {
    #[doc(hidden)]
    pub fn init() {
        STACK.set(StackImpl::new());

        // Spawn the network thread
        Scheduler::spawn(move || {
            StackImpl::network_thread();
        });
    }

    /// Returns interfaces registered in the network stack
    pub fn interfaces<'a>() -> Interfaces<'a> {
        Interfaces {
            imp: STACK.as_ref(),
        }
    }

    #[inline]
    /// Enqueue a packet inside the `rx_queue` and notify `rx_wait`.
    ///
    /// Returns false if the `rx_queue` is full and therefor was not enqueued,
    /// true otherwise.
    ///
    /// Note: This is safe to be called from interrupt context. Indeed the
    /// `rx_queue` is not resizable so no allocation will be performed by this
    /// function.
    pub fn enqueue_rx_packet(intf: Weak<RwLock<Interface>>,
                             packet: Packet) -> bool {
        STACK.as_mut().enqueue_rx_packet(intf, packet)
    }
}

/// Reference over the list of interfaces detected
pub struct Interfaces<'a> {
    imp: GlobalCellRef<'a, StackImpl>,
}

impl<'a> Interfaces<'a> {
    #[inline]
    /// Returns the number of interfaces detected
    pub fn count(&self) -> usize {
        self.imp.interfaces.len()
    }

    #[inline]
    /// Returns the list of interfaces as an immutable slice
    pub fn as_slice(&self) -> &[Arc<RwLock<Interface>>] {
        &self.imp.interfaces[..]
    }
}

pub struct StackImpl {
    /// Interfaces registered
    interfaces: Vec<Arc<RwLock<Interface>>>,
    /// Contains packets to be processed
    rx_queue: InterruptSpinLock<VecDeque<(Weak<RwLock<Interface>>, Packet)>>,
    /// Used to wait for packet to arrive in the rx_queue
    rx_wait: WaitQueue,
}

// rx_wait is Sync
// rx_queue is protected by a spin lock
unsafe impl Sync for StackImpl {}

impl StackImpl {
    pub fn network_thread() {
        // Get a mutable reference from the start so that we don't have to use
        // the GlobalCell for every packet.
        let imp: &mut StackImpl = &mut *STACK.as_mut();

        loop {
            let pkt_opt = imp.rx_queue.lock().pop_front();

            match pkt_opt {
                None => {
                    imp.refresh_interfaces();
                    // No packet to process => wait for one to come
                    wait_event!(imp.rx_wait, !imp.rx_queue.lock().is_empty());
                    imp.refresh_interfaces();
                }
                Some((weak_intf, pkt)) => {
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

    /// Call refresh on every registered interface
    fn refresh_interfaces(&mut self) {
        for intf in &mut self.interfaces {
            intf.write().refresh();
        }
    }

    pub fn enqueue_rx_packet(&mut self, intf: Weak<RwLock<Interface>>,
                             packet: Packet) -> bool {
        let mut locked_rx_queue = self.rx_queue.lock();

        if locked_rx_queue.len() == MAX_QUEUE_SIZE {
            // Queue is full, we don't want to cause a reallocation in
            // interruption context. So we don't enqueue the packet
            return false;
        }

        // Enqueue the packet
        locked_rx_queue.push_back((intf, packet));

        // Wake up network thread
        self.rx_wait.unblock();

        true
    }
}
