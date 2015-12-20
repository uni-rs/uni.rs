#![feature(asm)]
#![feature(alloc)]
#![feature(fnbox)]
#![feature(unique)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(collections)]
#![feature(macro_reexport)]
#![feature(core_intrinsics)]
#![no_std]

#[cfg(not(test))]
#[macro_reexport(vec)]
extern crate collections;

#[cfg(not(test))]
extern crate alloc;

#[cfg(not(test))]
pub use collections::*;

extern crate rlibc;
extern crate spin;

#[cfg(not(test))]
extern crate alloc_uni;
extern crate intrusive;

#[cfg(not(test))]
#[doc(hidden)]
pub mod allocator {
    pub use alloc_uni;
    pub unsafe fn init(region_start: usize, region_size: usize) {
        alloc_uni::init(region_start, region_size);
    }
}

#[macro_use]
pub mod io;
#[cfg(not(test))]
#[macro_use]
pub mod thread;

pub mod hal;
pub mod sync;

#[doc(hidden)]
pub mod utils;
