use io::lazy::Lazy;
use io::{Read, Write, Result};

use sync::Arc;

use sync::spin::{InterruptSpinLock, InterruptSpinGuard};

pub fn stdout() -> Stdout {
    static STDOUT: Lazy<InterruptSpinLock<&'static mut ::hal::Console>> = Lazy::new(stdout_init);

    return Stdout {
        inner: unsafe { STDOUT.get() },
    };

    fn stdout_init() -> Arc<InterruptSpinLock<&'static mut ::hal::Console>> {
        Arc::new(InterruptSpinLock::new(::hal::console()))
    }
}

pub struct Stdout {
    inner: Arc<InterruptSpinLock<&'static mut ::hal::Console>>,
}

impl Stdout {
    pub fn lock(&self) -> StdoutLock {
        StdoutLock {
            lock: self.inner.lock(),
        }
    }
}

pub struct StdoutLock<'a> {
    lock: InterruptSpinGuard<'a, &'static mut ::hal::Console>,
}

impl<'a> Write for StdoutLock<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.lock.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.lock.flush()
    }
}

pub fn stdin() -> Stdin {
    static STDIN: Lazy<InterruptSpinLock<&'static mut ::hal::Console>> = Lazy::new(stdin_init);

    return Stdin {
        inner: unsafe { STDIN.get() },
    };

    fn stdin_init() -> Arc<InterruptSpinLock<&'static mut ::hal::Console>> {
        Arc::new(InterruptSpinLock::new(::hal::console()))
    }
}

pub struct Stdin {
    inner: Arc<InterruptSpinLock<&'static mut ::hal::Console>>,
}

impl Stdin {
    pub fn lock(&self) -> StdinLock {
        StdinLock {
            lock: self.inner.lock(),
        }
    }
}

pub struct StdinLock<'a> {
    lock: InterruptSpinGuard<'a, &'static mut ::hal::Console>,
}

impl<'a> Read for StdinLock<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.lock.read(buf)
    }
}
