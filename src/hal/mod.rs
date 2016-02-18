//! Hardware Abstraction Layer
//!
//! The goal of this module is to abstract the machine/architecture so that
//! the API is the same no matter the underlying hardware.
//!
//! For now only Xen (x86/x86_64) is supported. A goal for the future would
//! be to be compatible with the virtio standard as well and support Xen arm

use io::{Read, Write, Result};

pub mod mmu;

pub use self::hw_imp::*;

/// Generic console wrapper
pub struct Console<'a>(HwConsoleType<'a>);

impl<'a> Read for Console<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl<'a> Write for Console<'a> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        self.0.flush()
    }
}

#[cfg(feature = "xen")]
pub mod xen;

#[cfg(feature = "xen")]
mod hw_imp {
    use super::xen;
    use super::Console;

    use cell::GlobalCellMutRef;

    /// Abstracts the actual hardware console type
    pub type HwConsoleType<'a> = GlobalCellMutRef<'a, xen::console::Console>;

    #[inline]
    /// Unprotected access to the hardware console
    ///
    /// This *SHOULD NOT* be used as is.
    ///
    /// Instead use [`io::stdout()`][stdout] or [`io::stdin()`][stdin]
    ///
    /// [stdout]: ../io/fn.stdout.html
    /// [stdin]: ../io/fn.stdin.html
    pub fn console<'a>() -> Console<'a> {
        Console(xen::console::console())
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

    #[cfg(feature = "net")]
    /// Network device driver abstraction
    pub mod net {
        use vec::Vec;

        use hal::xen::net;

        use net::Interface;

        /// Type of the hardware interface.
        pub type HwInterface = net::XenNetDevice;

        /// Discover hardware configuration and return a list of existing
        /// interfaces
        pub fn discover() -> Vec<Interface> {
            net::discover()
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[path="x86"]
pub mod arch {
    pub mod mmu;
    pub mod defs;
    pub mod utils;
}
