//! Hardware Abstraction Layer
//!
//! The goal of this module is to abstract the machine/architecture so that
//! the API is the same no matter the underlying hardware.
//!
//! For now only Xen (x86/x86_64) is supported. A goal for the future would
//! be to be compatible with the virtio standard as well and support Xen arm

pub use self::hw_imp::*;
pub use self::arch_imp::*;

pub trait Console: ::io::Read + ::io::Write {}

#[cfg(feature = "xen")]
pub mod xen;

#[cfg(feature = "xen")]
mod hw_imp {
    use super::xen;

    #[inline]
    pub fn console<'a>() -> &'a mut super::Console {
        xen::console::console()
    }

    #[inline]
    /// Disable local interrupt delivery
    pub fn local_irq_disable() {
        xen::disable_upcalls();
    }

    #[inline]
    /// Enable local interrupt delivery
    pub fn local_irq_enable() {
        xen::enable_upcalls();
    }

    #[inline]
    /// Disable local interrupt delivery and return the previous state of
    /// interrupt delivery
    pub fn local_irq_save() -> usize {
        xen::disable_upcalls() as usize
    }

    #[inline]
    /// Restore a local interrupt delivery state
    pub fn local_irq_restore(state: usize) {
        xen::set_upcalls_state(state as u8);
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[path="arch/x86.rs"]
mod x86;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod arch {
    pub use super::x86::*;
}

mod arch_imp {
    pub mod intrinsics {
        pub use core::intrinsics::atomic_xchg;

        pub use hal::arch::atomic_set_bit;
        pub use hal::arch::atomic_clear_bit;

        pub use hal::arch::first_bit;

        pub use hal::arch::wmb;
    }

    pub mod defs {
        pub use hal::arch::PAGE_SIZE;
    }
}
