#![feature(asm)]
#![feature(alloc)]
#![feature(fnbox)]
#![feature(unique)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(collections)]
#![feature(macro_reexport)]
#![feature(core_intrinsics)]
#![feature(op_assign_traits)]
#![feature(augmented_assignments)]

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

#![cfg_attr(feature = "net", feature(str_char))]

#![no_std]

#[macro_use]
#[macro_reexport(vec)]
#[macro_reexport(format)]
extern crate collections;
extern crate alloc;

#[cfg(not(test))] extern crate alloc_uni;
extern crate rlibc;
extern crate spin;

extern crate intrusive;

pub use collections::*;

#[macro_use]
mod macros;

pub mod io;
pub mod hal;
pub mod ffi;
pub mod num;
pub mod cell;
pub mod sync;
#[doc(hidden)] pub mod utils;
#[cfg(not(test))] pub mod thread;

#[cfg(feature = "net")] pub mod net;
