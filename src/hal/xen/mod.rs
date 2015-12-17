use hal::intrinsics::wmb;

mod hypercall;

pub mod defs;

pub mod boot;
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

        wmb();
        shared_info.vcpu_info[0].evtchn_upcall_mask = 0;
        wmb();

        ret
    }
}

pub fn disable_upcalls() -> u8 {
    unsafe {
        let ret = shared_info.vcpu_info[0].evtchn_upcall_mask;

        shared_info.vcpu_info[0].evtchn_upcall_mask = 1;
        wmb();

        ret
    }
}

pub fn set_upcalls_state(state: u8) {
    unsafe {
        wmb();
        shared_info.vcpu_info[0].evtchn_upcall_mask = state;
        wmb();
    }
}
