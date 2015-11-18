#![feature(asm)]
#![feature(no_std)]
#![feature(const_fn)]
#![feature(core_str_ext)]
#![no_std]

mod barrier;
mod hypercall;

pub mod defs;

pub mod memory;
pub mod event;
pub mod sched;

pub mod console;

extern "C" {
    // This symbol must be present in code using libxen
    pub static shared_info: self::defs::SharedInfo;
}
