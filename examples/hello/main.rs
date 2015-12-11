#![feature(start)]
#![no_std]

#[macro_use]
extern crate uni;

use uni::string::String;

#[start]
fn main(_: isize, _: *const *const u8) -> isize {
    let mut s = String::new();

    s.push_str("Hello");
    s.push_str(" ");
    s.push_str("World");

    println!("{}", s);

    0
}
