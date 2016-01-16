//! An UDP echo server

#![feature(start)]
#![no_std]

#[macro_use]
extern crate uni;

use uni::thread::Scheduler;

#[start]
fn main(_: isize, _: *const *const u8) -> isize {
    loop {
        Scheduler::schedule();
    }
}
