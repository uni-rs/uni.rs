//! Networking primitives for Uni.rs

use sync::Arc;
use sync::spin::RwLock;

use cell::{GlobalCell, GlobalCellRef};

use thread::Scheduler;

use self::imp::StackImpl;

mod imp;

mod pkt;
mod intf;
mod conn;

mod eth;

pub mod defs;

pub use self::pkt::Packet;
pub use self::intf::{Interface, V4Configuration};

static STACK: GlobalCell<StackImpl> = GlobalCell::new();

/// Uni.rs network stack
pub struct Stack;

impl Stack {
    #[doc(hidden)]
    pub fn init() {
        STACK.set(StackImpl::new());

        // Spawn the network thread
        Scheduler::spawn(|| {
            StackImpl::network_thread(&mut *STACK.as_mut());
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
    pub fn enqueue_rx_packet(packet: Packet) -> bool {
        STACK.as_mut().enqueue_rx_packet(packet)
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
        self.imp.interfaces().len()
    }

    #[inline]
    /// Returns the list of interfaces as an immutable slice
    pub fn as_slice(&self) -> &[Arc<RwLock<Interface>>] {
        self.imp.interfaces()
    }
}
