//! Hardware Abstraction Layer
//!
//! The goal of this module is to abstract the machine/architecture so that
//! the API is the same no matter the underlying hardware.
//!
//! For now only Xen (x86/x86_64) is supported. A goal for the future would
//! be to be compatible with the virtio standard as well and support Xen arm

pub use self::hw_imp::*;
pub use self::arch_imp::*;

pub mod xen;

mod hw_imp {
    use super::xen;

    #[inline]
    pub fn local_irq_disable() {
        xen::disable_upcalls();
    }

    #[inline]
    pub fn local_irq_enable() {
        xen::enable_upcalls();
    }

    #[inline]
    pub fn local_irq_save() -> usize {
        xen::disable_upcalls() as usize
    }

    #[inline]
    pub fn local_irq_restore(state: usize) {
        xen::set_upcalls_state(state as u8);
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[path="arch/x86.rs"]
mod arch;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod arch_imp {
    pub mod intrinsics {
        pub use core::intrinsics::atomic_xchg;

        pub use hal::arch::atomic_set_bit;
        pub use hal::arch::atomic_clear_bit;

        pub use hal::arch::first_bit;

        pub use hal::arch::wmb;
    }
}
