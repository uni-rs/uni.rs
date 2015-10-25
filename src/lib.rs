#![feature(no_std, lang_items, asm, const_fn)]
#![feature(core_str_ext)]
#![no_std]

#[macro_use]
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

    println!("Uni.rs is booting");
}

#[no_mangle]
pub fn uni_rust_entry() -> ! {
    let app_ret;

    init();

    unsafe {
        app_ret = main(0, core::ptr::null());
    }

    xen::console::console().flush();

    xen::sched::poweroff(app_ret as arch::defs::Ulong);

    panic!("Failed to poweroff the machine !");
}
