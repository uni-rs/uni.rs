#![feature(const_fn)]
#![no_std]
#![no_builtins]

#[macro_use]
extern crate uni;

pub mod event;
pub mod arch;
pub mod libc;

extern {
    fn main(_: isize, _: *const *const u8) -> isize;
}

use uni::thread::Scheduler;

// 8KB
const STACK_SIZE: usize = 8192;

#[no_mangle]
#[allow(non_upper_case_globals)]
#[link_section=".stack"]
pub static rust_stack: [u8; STACK_SIZE] = [0; STACK_SIZE];

#[no_mangle]
pub extern "C" fn uni_rust_entry() -> ! {
    self::arch::init();

    println!("Uni.rs is booting");

    // Memory initialization is unsafe
    unsafe {
        let (heap_start, heap_size) = arch::init_memory();

        uni::allocator::init(heap_start, heap_size);
    }

    event::init();

    unsafe {
        uni::console::console().init_input();
    }

    uni::hal::xen::enable_upcalls();

    println!("Creating main thread");

    // Spawn main thread
    Scheduler::spawn(|| {
        let app_ret = unsafe {
            main(0, core::ptr::null())
        };

        uni::hal::xen::disable_upcalls();

        uni::console::console().flush();

        uni::hal::xen::sched::poweroff(app_ret as uni::hal::xen::defs::Ulong);

        panic!("Failed to poweroff the machine !");
    });

    println!("Starting scheduler");

    Scheduler::schedule();

    unreachable!();
}
