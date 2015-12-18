//! Hardware Abstraction Layer
//!
//! The goal of this module is to abstract the machine/architecture so that
//! the API is the same no matter the underlying hardware.
//!
//! For now only Xen (x86/x86_64) is supported. A goal for the future would
//! be to be compatible with the virtio standard as well and support Xen arm

pub use self::hw_imp::*;
pub use self::arch_imp::*;

/// Trait implemented by the hardware console
pub trait Console: ::io::Read + ::io::Write {}

#[cfg(feature = "xen")]
pub mod xen;

#[cfg(feature = "xen")]
mod hw_imp {
    use super::xen;

    #[inline]
    /// Unprotected access to the hardware console
    ///
    /// This *SHOULD NOT* be used as is.
    ///
    /// Instead use [`io::stdout()`][stdout].
    ///
    /// [stdout]: ../io/fn.stdout.html
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

    /// Work with the state of the application
    pub mod app {
        use hal::xen;

        /// Block the application until an interruption arrives
        #[inline]
        pub fn block() {
            xen::sched::block();
        }

        /// Crash the application
        ///
        /// On certain platform this might be equivalent to
        /// [`exit()`][exit]
        ///
        /// [exit]: fn.exit.html
        #[inline]
        pub fn crash() {
            xen::sched::crash();
        }

        /// Exit the application
        ///
        /// This will also power off the system.
        ///
        /// Note that the error code might be ignored on certain platforms
        #[inline]
        pub fn exit(code: isize) {
            xen::sched::poweroff(code as xen::defs::Ulong);
        }
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

    /// Various definitions related to the architecture
    pub mod defs {
        pub use hal::arch::PAGE_SIZE;
    }
}
