#![feature(no_std, lang_items, asm)]
#![no_std]

pub mod xen;
pub mod arch;
pub mod utils;

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
    arch::init();
}

#[no_mangle]
pub fn uni_rust_entry() {
    init();

    unsafe {
        let _ = main(0, core::ptr::null());
    }
}

