//! Networking primitives for Uni.rs

use cell::GlobalCell;

use thread::Scheduler;

mod imp;

mod pkt;
mod intf;
pub mod conn;

mod eth;

pub mod defs;

pub use self::imp::{Instance, InstanceWeak};

pub use self::pkt::{
    Packet,
    Builder as PacketBuilder,
    Formatter as PacketFormatter,
};

pub use self::intf::{Interface, InterfaceWeak, InterfaceRaw, V4Configuration};

static STACK: GlobalCell<Instance> = GlobalCell::new();

/// Uni.rs network stack
pub struct Stack;

impl Stack {
    #[doc(hidden)]
    pub fn init() {
        STACK.set(Instance::new());

        // Spawn the network thread
        Scheduler::spawn(|| {
            Instance::network_thread(STACK.as_ref().clone());
        });
    }

    /// Returns interfaces registered in the network stack
    pub fn instance() -> Instance {
        STACK.as_ref().clone()
    }
}
