//! Definition of types, traits, ... for I/O functionality

use core::result;

pub type Result<T> = result::Result<T, ()>;

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;
}
