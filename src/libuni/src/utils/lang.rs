use core::fmt;

use xen::sched;

use console;

#[cfg(not(test))]
#[lang = "stack_exhausted"]
pub fn stack_exhausted() -> ! {
    unimplemented!()
}

#[cfg(not(test))]
#[lang = "eh_personality"]
pub fn eh_personality() -> ! {
    unimplemented!()
}

#[cfg(not(test))]
#[lang = "panic_fmt"]
pub extern fn panic_impl(msg: fmt::Arguments, file: &'static str, line: u32) {
    println!("Panic at '{}:{}' with message '{}'", file, line, msg);
    console::console().flush();

    sched::crash();

    loop {}
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn _Unwind_Resume(_: *mut u8) -> ! {
    panic!("_Unwind_Resume called");
}
