#![feature(no_std, lang_items)]
#![no_std]

pub mod utils;

extern {
    fn main(_: isize, _: *const *const u8) -> isize;
}

#[no_mangle]
pub fn uni_rust_entry() {
    unsafe {
        let _ = main(0, core::ptr::null());
    }
}

#[lang = "stack_exhausted"] pub fn stack_exhausted() {}
#[lang = "eh_personality"] pub fn eh_personality() {}
#[lang = "panic_fmt"] pub fn panic_fmt() -> ! { loop {} }
