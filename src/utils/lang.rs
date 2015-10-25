use core::fmt::Arguments;

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
pub fn panic_fmt(_: Arguments, _: &(&'static str, u32)) -> ! {
    loop {}
}
