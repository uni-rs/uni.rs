use hal::xen::console::Console;
use hal::xen::defs::{ConsoleInterface, EvtchnPort};

use core::fmt::{Arguments, write};

use sync::spin::InterruptSpinLock;
use sync::spin::InterruptSpinGuard;

static mut CONSOLE: Option<InterruptSpinLock<Console>> = None;

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
        $crate::console::_print(format_args!($($arg)*));
    }
}

pub fn _print(fmt: Arguments) {
    let res = write(&mut *console(), fmt);

    if let Err(e) = res {
        panic!("Fail to print on the Xen console: {}", e);
    }
}

pub unsafe fn init(interface: *mut ConsoleInterface, port: EvtchnPort) {
    CONSOLE = Some(InterruptSpinLock::new(Console::new(interface, port)));
}

pub fn console<'a>() -> InterruptSpinGuard<'a, Console> {
    unsafe {
        CONSOLE.as_ref().expect("Console used before being initialized").lock()
    }
}
