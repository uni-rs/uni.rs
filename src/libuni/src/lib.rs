#![feature(no_std)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(collections)]
#![no_std]

#[macro_use]
extern crate collections;

pub use collections::*;

#[cfg(test)]
extern crate std;

extern crate spin;
extern crate alloc_uni;

extern crate xen;

pub mod alloc {
    pub use alloc_uni;
    pub unsafe fn init(region_start: usize, region_size: usize) {
        alloc_uni::init(region_start, region_size);
    }
}

#[macro_use]
pub mod console;

pub mod utils;
