use ::arch::defs::Ulong;

extern {
    static hypercall_page: u8;
}

pub unsafe fn hypercall(id: u32, arg1: Ulong, arg2: Ulong, arg3: Ulong,
                        arg4: Ulong, arg5: Ulong) -> Ulong {
    let ret : Ulong;
    let mut hypercall_addr: *const u8 = &hypercall_page;

    hypercall_addr = hypercall_addr.offset(id as isize * 32);

    asm!("call *$1"
         : "={rax}" (ret)
         : "r" (hypercall_addr), "{rdi}" (arg1), "{rsi}" (arg2),
           "{rdx}" (arg3), "{r10}" (arg4), "{r8}" (arg5)
         : "memory"
         : "volatile"
         );

    ret
}
