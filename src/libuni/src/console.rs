use xen::console::Console;
use xen::defs::{ConsoleInterface, EvtchnPort};

use core::fmt::{Arguments, write};

use spin::Mutex as SpinLock;
use spin::MutexGuard as SpinGuard;

static mut CONSOLE: Option<SpinLock<Console>> = None;

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
    CONSOLE = Some(SpinLock::new(Console::new(interface, port)));
}

pub fn console<'a>() -> SpinGuard<'a, Console> {
    unsafe {
        CONSOLE.as_mut().expect("Console used before being initialized").lock()
    }
}
