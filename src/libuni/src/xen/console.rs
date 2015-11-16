use core::ops::Deref;
use core::ops::DerefMut;

use core::fmt::{Write, Result, Arguments, write};

use core::mem::size_of;

use os::lock::{SpinLock, SpinGuard};

use xen::event;
use xen::sched;

static CONSOLE: SpinLock<Console> = SpinLock::new(Console::new());

type ConsRingIdx = u32;

#[macro_export]
macro_rules! println {
    ($fmt:expr) => {
        print!(concat!($fmt, "\r\n"))
    };
    ($fmt:expr, $($arg:tt)*) => {
        print!(concat!($fmt, "\r\n"), $($arg)*)
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::xen::console::_print(format_args!($($arg)*));
    }
}

pub fn _print(fmt: Arguments) {
    let res = write(&mut *console(), fmt);

    if let Err(e) = res {
        panic!("Fail to print on the Xen console: {}", e);
    }
}

pub fn console<'a>() -> SpinGuard<'a, Console> {
    CONSOLE.lock()
}

#[repr(C)]
pub struct ConsoleInterface {
    input: [u8; 1024],
    output: [u8; 2048],
    in_cons: ConsRingIdx,
    in_prod: ConsRingIdx,
    out_cons: ConsRingIdx,
    out_prod: ConsRingIdx,
}

pub struct Console {
    interface: *mut ConsoleInterface,
    port: event::EvtchnPort,
}

impl Console {
    pub const fn new() -> Self {
        Console {
            interface: 0 as *mut ConsoleInterface,
            port: 0,
        }
    }

    pub fn set_interface(&mut self, interface: *mut ConsoleInterface) {
        self.interface = interface
    }

    pub fn set_port(&mut self, port: event::EvtchnPort) {
        self.port = port
    }

    fn is_output_full(&self) -> bool {
        let data: ConsRingIdx;

        data = self.out_prod - self.out_cons;

        // size_of output field
        (data as usize) >= size_of::<[u8; 2048]>()
    }

    fn out_idx(&self) -> usize {
        let size_of_output = (size_of::<[u8; 2048]>() - 1) as u32;

        (self.out_prod & size_of_output) as usize
    }

    pub fn flush(&self) -> () {
        while self.out_cons < self.out_prod {
            sched::yield_cpu();
        }
    }
}

impl Deref for Console {
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

impl DerefMut for Console {
    #[inline]
    fn deref_mut(&mut self) -> &mut ConsoleInterface {
        // See deref
        unsafe {
            &mut (*self.interface)
        }
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> Result {

        for c in s.as_bytes() {
            while self.is_output_full() {
                event::send(self.port);
            }

            let index = self.out_idx();

            self.output[index] = *c;
            ::arch::barrier::wmb();

            self.out_prod += 1;
        }

        event::send(self.port);

        Ok(())
    }
}
