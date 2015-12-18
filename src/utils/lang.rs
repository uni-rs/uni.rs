use core::fmt;

use io::Write;

use hal::{app, console};

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
    // Use raw console to be sure to not be locked
    console().write_fmt(format_args!("Panic at '{}:{}' with message '{}'\n\r",
                                     file, line, msg)).unwrap();
    console().flush().unwrap();

    app::crash();

    loop {}
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn _Unwind_Resume(_: *mut u8) -> ! {
    panic!("_Unwind_Resume called");
}
