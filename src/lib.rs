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

#[no_mangle]
pub fn uni_rust_entry() {
    unsafe {
        let _ = main(0, core::ptr::null());
    }
}

#[lang = "stack_exhausted"] pub fn stack_exhausted() {}
#[lang = "eh_personality"] pub fn eh_personality() {}
#[lang = "panic_fmt"] pub fn panic_fmt() -> ! { loop {} }
