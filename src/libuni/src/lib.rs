#![feature(no_std)]
#![feature(const_fn)]
#![feature(lang_items)]
#![no_std]

extern crate spin;

extern crate xen;

#[cfg(test)]
extern crate std;

extern crate alloc_uni;

pub mod alloc {
    pub use alloc_uni;
    pub unsafe fn init(region_start: usize, region_size: usize) {
        alloc_uni::init(region_start, region_size);
    }
}

#[macro_use]
pub mod console;

pub mod utils;
