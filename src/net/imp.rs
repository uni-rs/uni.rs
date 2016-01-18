use sync::Arc;

use vec_deque::VecDeque;

use cell::GlobalCell;

use sync::spin::{InterruptSpinLock, RwLock};

use thread::WaitQueue;

use net::Interface;

use net::Packet;

const MAX_QUEUE_SIZE: usize = 512;

pub static STACK: GlobalCell<StackImpl> = GlobalCell::new();

/// Uni.rs network stack
pub struct Stack;

impl Stack {
    #[doc(hidden)]
    pub fn init() {
        STACK.set(StackImpl::new());
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
    pub fn enqueue_rx_packet(intf: *const Arc<RwLock<Interface>>,
                             packet: Packet) -> bool {
        STACK.as_mut().enqueue_rx_packet(intf, packet)
    }
}

pub struct StackImpl {
    /// Contains packets to be processed
    rx_queue: InterruptSpinLock<VecDeque<(*const Arc<RwLock<Interface>>, Packet)>>,
    /// Used to wait for packet to arrive in the rx_queue
    rx_wait: WaitQueue,
}

// rx_wait is Sync
// rx_queue is protected by a spin lock
unsafe impl Sync for StackImpl {}

impl StackImpl {
    pub fn new() -> Self {
        StackImpl {
            rx_queue: InterruptSpinLock::new(VecDeque::with_capacity(MAX_QUEUE_SIZE)),
            rx_wait: WaitQueue::new(),
        }
    }

    pub fn enqueue_rx_packet(&mut self, intf: *const Arc<RwLock<Interface>>,
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
