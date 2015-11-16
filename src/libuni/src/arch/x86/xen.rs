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
         : "={eax}" (ret)
         : "{eax}" (hypercall_addr), "{ebx}" (arg1), "{ecx}" (arg2),
           "{edx}" (arg3), "{esi}" (arg4), "{edi}" (arg5)
         : "memory"
         : "volatile"
         );

    ret
}
