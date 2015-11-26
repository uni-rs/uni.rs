use core::ops::Deref;
use core::ops::DerefMut;

use core::fmt::{Write, Result};

use core::mem::size_of;

use defs::{ConsoleInterface, ConsRingIdx, EvtchnPort};

use event::send;
use sched::yield_cpu;

use intrinsics::wmb;

pub struct Console {
    interface: *mut ConsoleInterface,
    port: EvtchnPort,
}

impl Console {
    pub unsafe fn new(interface: *mut ConsoleInterface,
                      port: EvtchnPort) -> Self {
        Console {
            interface: interface,
            port: port,
        }
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
            yield_cpu();
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
                send(self.port);
            }

            let index = self.out_idx();

            self.output[index] = *c;
            wmb();

            self.out_prod += 1;
        }

        send(self.port);

        Ok(())
    }
}
