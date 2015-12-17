#![feature(asm)]
#![feature(alloc)]
#![feature(fnbox)]
#![feature(unique)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(collections)]
#![feature(core_intrinsics)]
#![no_std]

#[macro_use]
#[cfg(not(test))]
extern crate collections;

#[cfg(not(test))]
pub use collections::*;

extern crate spin;
extern crate intrusive;

#[cfg(not(test))]
extern crate alloc_uni;

#[cfg(not(test))]
extern crate alloc;

#[cfg(not(test))]
#[doc(hidden)]
pub mod allocator {
    pub use alloc_uni;
    pub unsafe fn init(region_start: usize, region_size: usize) {
        alloc_uni::init(region_start, region_size);
    }
}

#[macro_use]
pub mod console;

pub mod hal;

#[cfg(not(test))]
pub mod thread;
pub mod sync;

#[doc(hidden)]
pub mod utils;
