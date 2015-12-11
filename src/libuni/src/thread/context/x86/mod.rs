use super::WrapperFn;

#[repr(C)]
#[derive(Debug)]
pub struct Registers {
    eax: u32, // 0
    ebx: u32, // 4
    ecx: u32, // 8
    edx: u32, // 12
    eip: u32, // 16
    ebp: u32, // 20
    esp: u32, // 24
    esi: u32, // 28
    edi: u32, // 32
    eflags: u32, // 36
}

impl Registers {
    pub const unsafe fn empty() -> Self {
        Registers {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
            eip: 0,
            ebp: 0,
            esp: 0,
            esi: 0,
            edi: 0,
            eflags: 0,
        }
    }

    pub unsafe fn new(wrapper: WrapperFn, f: *mut u8,
                      stack_top: *mut u8) -> Self {
        // wrapper is the function that is going to be called when we switch
        // for the first time to this set of register. This function expects
        // one argument which is the pointer over the actual closure called.
        // One x86 the first argument of a function is located on the stack.
        // So here is the layout of the stack:
        //
        // +--------------------+ <- stack_top
        // |         f          |
        // +--------------------+
        // | 0 (fake ret addr)  |
        // +--------------------+ <- esp
        // |                    |
        // |        ...         |
        // |                    |
        let esp = stack_top.offset(-8);

        *(stack_top.offset(-4) as *mut u32) = f as u32;
        *(stack_top.offset(-8) as *mut u32) = 0;

        Registers {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
            eip: wrapper as u32,
            ebp: 0,
            esp: esp as u32,
            esi: 0,
            edi: 0,
            eflags: 0,
        }
    }
}

