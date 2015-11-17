#![feature(no_std)]
#![no_std]

#[macro_use]
extern crate uni;

extern crate xen;

extern {
    fn main(_: isize, _: *const *const u8) -> isize;
}

// 8KB
const STACK_SIZE: usize = 8192;

#[no_mangle]
#[allow(non_upper_case_globals)]
#[link_section=".stack"]
pub static rust_stack: [u8; STACK_SIZE] = [0; STACK_SIZE];

fn init() {
    uni::arch::init();

    println!("Uni.rs is booting");

    // Memory initialization is unsafe
    unsafe {
        let (heap_start, heap_size) = uni::arch::init_memory();

        uni::alloc::init(heap_start, heap_size);
    }
}

#[no_mangle]
pub extern "C" fn uni_rust_entry() -> ! {
    let app_ret;

    init();

    unsafe {
        app_ret = main(0, core::ptr::null());
    }

    uni::console::console().flush();

    // xen::sched::poweroff(app_ret as xen::defs::Ulong);

    panic!("Failed to poweroff the machine !");
}

