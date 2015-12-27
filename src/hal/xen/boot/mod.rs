use hal::arch::defs::PAGE_SIZE;

mod x86;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use self::x86::*;

const STACK_SIZE: usize = 2 * PAGE_SIZE;

#[no_mangle]
#[allow(non_upper_case_globals)]
#[link_section=".stack"]
pub static rust_stack: [u8; STACK_SIZE] = [0; STACK_SIZE];
