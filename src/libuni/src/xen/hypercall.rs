use ::arch::defs::Ulong;

pub enum HyperCalls {
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

pub fn hypercall0(id: HyperCalls) -> Ulong {
    unsafe {
        ::arch::xen::hypercall(id as u32, 0, 0, 0, 0, 0)
    }
}

pub fn hypercall1(id: HyperCalls, arg1: Ulong) -> Ulong {
    unsafe {
        ::arch::xen::hypercall(id as u32, arg1, 0, 0, 0, 0)
    }
}

pub fn hypercall2(id: HyperCalls, arg1: Ulong, arg2: Ulong) -> Ulong {
    unsafe {
        ::arch::xen::hypercall(id as u32, arg1, arg2, 0, 0, 0)
    }
}

pub fn hypercall3(id: HyperCalls, arg1: Ulong, arg2: Ulong,
                  arg3: Ulong) -> Ulong {
    unsafe {
        ::arch::xen::hypercall(id as u32, arg1, arg2, arg3, 0, 0)
    }
}

pub fn hypercall4(id: HyperCalls, arg1: Ulong, arg2: Ulong, arg3: Ulong,
                  arg4: Ulong) -> Ulong {
    unsafe {
        ::arch::xen::hypercall(id as u32, arg1, arg2, arg3, arg4, 0)
    }
}

pub fn hypercall5(id: HyperCalls, arg1: Ulong, arg2: Ulong, arg3: Ulong,
                  arg4: Ulong, arg5: Ulong) -> Ulong {
    unsafe {
        ::arch::xen::hypercall(id as u32, arg1, arg2, arg3, arg4, arg5)
    }
}
