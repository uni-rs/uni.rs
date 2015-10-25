use core::ops::Deref;
use core::ops::DerefMut;

use xen::event;

#[allow(dead_code)]
static mut CONS: InnerConsole = InnerConsole::new();

type ConsRingIdx = u32;

#[repr(C)]
pub struct ConsoleInterface {
    input: [u8; 1024],
    output: [u8; 2048],
    in_cons: ConsRingIdx,
    in_prod: ConsRingIdx,
    out_cons: ConsRingIdx,
    out_prod: ConsRingIdx,
}

struct InnerConsole {
    pub interface: *mut ConsoleInterface,
    pub port: event::EvtchnPort,
}

impl InnerConsole {
    pub const fn new() -> InnerConsole {
        InnerConsole {
            interface: 0 as *mut ConsoleInterface,
            port: 0,
        }
    }
}

impl Deref for InnerConsole {
    type Target = ConsoleInterface;

    #[inline]
    fn deref(&self) -> &ConsoleInterface {
        // This isn't totally safe. If the interface has not been initialized
        // this would cause some problems. For this to be safe, the console
        // MUST be initialized before being used
        unsafe {
            &(*self.interface)
        }
    }
}

impl DerefMut for InnerConsole {
    #[inline]
    fn deref_mut(&mut self) -> &mut ConsoleInterface {
        // See deref
        unsafe {
            &mut (*self.interface)
        }
    }
}
