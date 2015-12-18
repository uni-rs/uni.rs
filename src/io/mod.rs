//! Definition of types, traits, ... for I/O functionality

use core::fmt;
use core::result;

mod lazy;
mod stdio;

pub use self::stdio::{stdin, stdout};
pub use self::stdio::{Stdin, Stdout, StdinLock, StdoutLock};

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
        $crate::io::_print(format_args!($($arg)*));
    }
}

#[doc(hidden)]
pub fn _print(fmt: fmt::Arguments) {
    let out = stdout();
    let mut out_locked = out.lock();
    let res = out_locked.write_fmt(fmt);

    if let Err(..) = res {
        panic!("Fail to print on the Xen console");
    }
}

pub type Result<T> = result::Result<T, ()>;

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;

    fn write_fmt(&mut self, fmt: fmt::Arguments) -> Result<()> {
        struct Adaptor<'a, T: ?Sized + 'a> {
            inner: &'a mut T,
        }

        impl<'a, T: ?Sized + Write> fmt::Write for Adaptor<'a, T> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                match self.inner.write(s.as_bytes()) {
                    Ok(..) => Ok(()),
                    Err(..) => Err(fmt::Error),
                }
            }
        }

        let mut tmp = Adaptor {
            inner: self,
        };

        match fmt::write(&mut tmp, fmt) {
            Ok(..) => Ok(()),
            Err(..) => Err(()),
        }
    }
}
