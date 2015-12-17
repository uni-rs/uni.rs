use hal::xen::defs::Ulong;

#[allow(dead_code)]
pub enum HypercallKind {
    SetTrapTable = 0,
    MmuUpdate = 1,
    SetGdt = 2,
    StackSwitch = 3,
    SetCallbacks = 4,
    FpuTaskSwitch = 5,
    SchedOpCompat = 6,
    PlatformOp = 7,
    SetDebugReg = 8,
    GetDebugReg = 9,
    UpdateDescriptor = 10,
    MemoryOp = 12,
    MultiCall = 13,
    UpdateVaMapping = 14,
    SetTimerOp = 15,
    EventChannelOpCompat = 16,
    XenVersion = 17,
    ConsoleIo = 18,
    PhysdevOpCompat = 19,
    GrantTableOp = 20,
    VmAssist = 21,
    UpdateVaMappingOtherdomain = 22,
    Iret = 23,
    VcpuOp = 24,
    SetSegmentBase = 25,
    MmuextOp = 26,
    XsmOp = 27,
    NmiOp = 28,
    SchedOp = 29,
    CallbackOp = 30,
    XenoprofOp = 31,
    EventChannelOp = 32,
    PhysdevOp = 33,
    HvmOp = 34,
    Sysctl = 35,
    Domctl = 36,
    KexecOp = 37,
    TmemOp = 38,
    XcReservedOp = 39,
    XenpmuOp = 40,
}

#[allow(dead_code)]
#[inline]
pub unsafe fn hypercall0(id: HypercallKind) -> Ulong {
    hypercall(id as u32, 0, 0, 0, 0, 0)
}

#[allow(dead_code)]
#[inline]
pub unsafe fn hypercall1(id: HypercallKind, arg1: Ulong) -> Ulong {
    hypercall(id as u32, arg1, 0, 0, 0, 0)
}

#[inline]
pub unsafe fn hypercall2(id: HypercallKind, arg1: Ulong, arg2: Ulong) -> Ulong {
    hypercall(id as u32, arg1, arg2, 0, 0, 0)
}

#[inline]
pub unsafe fn hypercall3(id: HypercallKind, arg1: Ulong, arg2: Ulong,
                         arg3: Ulong) -> Ulong {
    hypercall(id as u32, arg1, arg2, arg3, 0, 0)
}

#[inline]
pub unsafe fn hypercall4(id: HypercallKind, arg1: Ulong, arg2: Ulong,
                         arg3: Ulong, arg4: Ulong) -> Ulong {
    hypercall(id as u32, arg1, arg2, arg3, arg4, 0)
}

#[allow(dead_code)]
#[inline]
pub unsafe fn hypercall5(id: HypercallKind, arg1: Ulong, arg2: Ulong,
                         arg3: Ulong, arg4: Ulong, arg5: Ulong) -> Ulong {
    hypercall(id as u32, arg1, arg2, arg3, arg4, arg5)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern {
    static hypercall_page: u8;
}

#[cfg(target_arch = "x86")]
unsafe fn hypercall(id: u32, arg1: Ulong, arg2: Ulong, arg3: Ulong,
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

#[cfg(target_arch = "x86_64")]
unsafe fn hypercall(id: u32, arg1: Ulong, arg2: Ulong, arg3: Ulong,
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
