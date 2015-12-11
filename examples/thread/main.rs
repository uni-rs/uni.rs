#![feature(start)]
#![no_std]

#[macro_use]
extern crate uni;

use uni::thread::{Thread, Scheduler};

#[start]
fn main(_: isize, _: *const *const u8) -> isize {
    let thread = Thread::spawn(|| {
        for i in 0..10 {
            println!("Loop: {}", i);

            Scheduler::schedule();
        }
    });

    Scheduler::ready(thread);

    for _ in 0..10 {
        println!("Main thread");

        Scheduler::schedule();
    }

    println!("Before");

    // Cause the destruction of the thread
    Scheduler::schedule();

    println!("After");

    // We are the last thread remaining, should do nothing
    Scheduler::schedule();

    println!("End!");

    0
}
