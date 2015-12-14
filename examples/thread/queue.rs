#![feature(start)]
#![no_std]

#[macro_use]
extern crate uni;

use uni::sync::Arc;

use uni::thread::{Thread, Scheduler, WaitQueue};

fn spawn_t1(queue: Arc<WaitQueue>) {
    let t1 = Thread::spawn(move || {
        println!("T1: Block");

        queue.block();

        println!("T1: Unblocked");
        println!("T1: Block");

        queue.block();

        println!("T1: Unblocked");
    });

    Scheduler::ready(t1);
}

fn spawn_t2(queue: Arc<WaitQueue>) {
    let t2 = Thread::spawn(move || {
        println!("T2: Gonna block");

        queue.block();

        println!("T2: Unblocked");
    });

    Scheduler::ready(t2);
}

#[start]
fn main(_: isize, _: *const *const u8) -> isize {
    let queue = Arc::new(WaitQueue::new());

    spawn_t1(queue.clone());
    spawn_t2(queue.clone());

    println!("Main: Start thread");

    // Run the threads
    Scheduler::schedule();

    println!("Main: Gonna unblock T1");

    // Unblock t1
    queue.unblock();

    println!("Main: T1 Unblocked");

    // Run t1
    Scheduler::schedule();

    println!("Main: Gonna unblock all threads");

    queue.unblock_all();

    // Run threads
    Scheduler::schedule();

    println!("Main: Done");

    // Threads done, this should do nothing
    Scheduler::schedule();

    println!("Main: End");

    0
}
