#![feature(no_std, lang_items, asm, const_fn)]
#![feature(core_str_ext)]
#![no_std]

extern crate rlibc;
extern crate heap;

#[cfg(test)]
extern crate std;

#[macro_use]
pub mod xen;

pub mod os;
pub mod arch;
pub mod utils;

pub mod alloc;
