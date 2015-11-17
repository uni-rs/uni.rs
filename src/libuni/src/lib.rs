#![feature(no_std)]
#![feature(const_fn)]
#![feature(lang_items)]
#![no_std]

extern crate heap;
extern crate xen;

#[cfg(test)]
extern crate std;

#[macro_use]
pub mod console;

pub mod os;
pub mod alloc;
pub mod utils;
