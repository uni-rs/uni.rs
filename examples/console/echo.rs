#![feature(start)]
#![no_std]

#[macro_use]
extern crate uni;

use uni::io::Read;
use uni::io::stdin;

use uni::string::String;

#[start]
fn main(_: isize, _: *const *const u8) -> isize {
    println!("Reading terminal to echo what you type");

    loop {
        let mut v = vec![0; 10];

        let stdin = stdin();
        let mut stdin_locked = stdin.lock();

        stdin_locked.read(&mut v[..]).unwrap();

        println!("Read: {}", String::from_utf8(v).unwrap());
    }
}
