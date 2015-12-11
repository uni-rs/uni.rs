use super::WrapperFn;

extern "C" {
    pub fn registers_switch(to_save: *mut Registers,
                            to_load: *const Registers);
}

#[repr(C)]
#[derive(Debug)]
pub struct Registers {
    rax: u64, // 0
    rbx: u64, // 8
    rcx: u64, // 16
    rdx: u64, // 24
    rip: u64, // 32
    rbp: u64, // 40
    rsp: u64, // 48
    rsi: u64, // 56
    rdi: u64, // 64
    r8: u64, // 72
    r9: u64, // 80
    r10: u64, // 88
    r11: u64, // 96
    r12: u64, // 104
    r13: u64, // 112
    r14: u64, // 120
    r15: u64, // 128
    rflags: u64, // 136
}

impl Registers {
    pub const unsafe fn empty() -> Self {
        Registers {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rip: 0,
            rbp: 0,
            rsp: 0,
            rsi: 0,
            rdi: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rflags: 0,
        }
    }

    pub unsafe fn new(wrapper: WrapperFn, f: *mut u8,
                      stack_top: *mut u8) -> Self {
        // wrapper is the function that is going to be called when we switch
        // for the first time to this set of register. This function expects
        // one argument which is the pointer over the actual closure called.
        // One x86_64 the first argument of a function is located in rsi

        Registers {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rip: wrapper as u64,
            rbp: 0,
            rsp: stack_top as u64,
            rsi: 0,
            rdi: f as u64,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rflags: 0,
        }
    }
}
