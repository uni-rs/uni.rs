#![feature(start, no_std)]
#![no_std]

#[macro_use]
extern crate uni;

#[start]
fn main(_: isize, _: *const *const u8) -> isize {
    println!("Hello World");

    0
}
