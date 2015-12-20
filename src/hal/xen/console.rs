use core::cmp;
use core::mem;
use core::ops::{Deref, DerefMut};

use io::{Read, Write, Result};

use thread::{Scheduler, WaitQueue};

use hal::xen::sched::yield_cpu;
use hal::xen::event::{dispatcher, send};
use hal::xen::defs::{ConsoleInterface, ConsRingIdx, EvtchnPort};

use hal::intrinsics::wmb;

static mut CONSOLE: Option<Console> = None;

pub fn console<'a>() -> &'a mut Console {
    unsafe {
        CONSOLE.as_mut().expect("Using console before initialization")
    }
}

pub unsafe fn init(interface: *mut ConsoleInterface, port: EvtchnPort) {
    CONSOLE = Some((Console::new(interface, port)));
}

pub struct Console {
    interface: *mut ConsoleInterface,
    port: EvtchnPort,
    queue: WaitQueue,
}

impl Console {
    pub fn console_callback(port: EvtchnPort, data: *mut u8) {
        let console = unsafe { &mut *(data as *mut Console) };
        let cons = console.in_cons;
        let prod = console.in_prod;

        if prod - cons > 0 {
            console.queue.unblock();
        }

        send(port);
    }

    pub unsafe fn new(interface: *mut ConsoleInterface,
                      port: EvtchnPort) -> Self {
        Console {
            interface: interface,
            port: port,
            queue: WaitQueue::new(),
        }
    }

    pub unsafe fn init_input(&mut self) {
        dispatcher().bind_port(self.port, Console::console_callback,
                               self as *mut Self as *mut u8);
        dispatcher().unmask_event(self.port);
        send(self.port);
    }

    fn is_output_full(&self) -> bool {
        let data: ConsRingIdx;

        data = self.out_prod - self.out_cons;

        // size_of output field
        (data as usize) >= mem::size_of::<[u8; 2048]>()
    }

    fn out_idx(&self) -> usize {
        let size_of_output = (mem::size_of::<[u8; 2048]>() - 1) as u32;

        (self.out_prod & size_of_output) as usize
    }

    fn in_idx(&self) -> usize {
        let size_of_input = (mem::size_of::<[u8; 1024]>() - 1) as u32;

        (self.in_cons & size_of_input) as usize
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

impl ::hal::Console for Console {}

impl Read for Console {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // More locked or atomic checks of the condition are not necessary
        // here. Indeed this Console is a "raw" console which means that it is
        // not thread safe. This safety will be guaranteed by the stdio
        // functions such as stdin, stdout, ...
        wait_event!(self.queue, self.in_cons - self.in_prod > 0);

        let i = 0;

        let size = cmp::min((self.in_cons - self.in_prod) as usize, buf.len());

        while i < buf.len() && self.in_cons < self.in_prod {
            let index = self.in_idx();

            buf[i] = self.input[index];

            self.in_cons += 1;

            wmb();
        }

        Ok(size)
    }
}

impl Write for Console {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        for c in buf {
            while self.is_output_full() {
                send(self.port);
            }

            let index = self.out_idx();

            self.output[index] = *c;
            wmb();

            self.out_prod += 1;
        }

        send(self.port);

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        while self.out_cons < self.out_prod {
            yield_cpu();
        }

        Ok(())
    }
}
