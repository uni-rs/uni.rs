use io::lazy::Lazy;
use io::{Read, Write, Result};

use sync::Arc;

use hal::Console;
use hal::console;

use sync::spin::{InterruptSpinLock, InterruptSpinGuard};


fn stdout_init() -> Arc<InterruptSpinLock<Console<'static>>> {
    Arc::new(InterruptSpinLock::new(console()))
}

pub fn stdout() -> Stdout {
    static STDOUT: Lazy<InterruptSpinLock<Console<'static>>> = Lazy::new(stdout_init);

    Stdout {
        inner: unsafe { STDOUT.get() },
    }
}

pub struct Stdout {
    inner: Arc<InterruptSpinLock<Console<'static>>>,
}

impl Stdout {
    pub fn lock(&self) -> StdoutLock {
        StdoutLock {
            lock: self.inner.lock(),
        }
    }
}

pub struct StdoutLock<'a> {
    lock: InterruptSpinGuard<'a, Console<'static>>,
}

impl<'a> Write for StdoutLock<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.lock.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.lock.flush()
    }
}


fn stdin_init() -> Arc<InterruptSpinLock<Console<'static>>> {
    Arc::new(InterruptSpinLock::new(console()))
}

pub fn stdin() -> Stdin {
    static STDIN: Lazy<InterruptSpinLock<Console<'static>>> = Lazy::new(stdin_init);

    Stdin {
        inner: unsafe { STDIN.get() },
    }
}

pub struct Stdin {
    inner: Arc<InterruptSpinLock<Console<'static>>>,
}

impl Stdin {
    pub fn lock(&self) -> StdinLock {
        StdinLock {
            lock: self.inner.lock(),
        }
    }
}

pub struct StdinLock<'a> {
    lock: InterruptSpinGuard<'a, Console<'static>>,
}

impl<'a> Read for StdinLock<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.lock.read(buf)
    }
}
