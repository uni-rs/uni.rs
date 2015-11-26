#![feature(asm)]
#![feature(no_std)]
#![feature(const_fn)]
#![feature(core_str_ext)]
#![no_std]

mod intrinsics;
mod hypercall;

pub mod defs;

pub mod memory;
pub mod event;
pub mod sched;

pub mod console;

pub mod arch;

extern "C" {
    // This symbol must be present in code using libxen
    pub static mut shared_info: self::defs::SharedInfo;
}

pub fn enable_upcalls() -> u8 {
    unsafe {
        let ret = shared_info.vcpu_info[0].evtchn_upcall_mask;

        self::intrinsics::wmb();
        shared_info.vcpu_info[0].evtchn_upcall_mask = 0;
        self::intrinsics::wmb();

        ret
    }
}

pub fn disable_upcalls() -> u8 {
    unsafe {
        let ret = shared_info.vcpu_info[0].evtchn_upcall_mask;

        shared_info.vcpu_info[0].evtchn_upcall_mask = 1;
        self::intrinsics::wmb();

        ret
    }
}

pub fn set_upcalls_state(state: u8) {
    unsafe {
        self::intrinsics::wmb();
        shared_info.vcpu_info[0].evtchn_upcall_mask = state;
        self::intrinsics::wmb();
    }
}
