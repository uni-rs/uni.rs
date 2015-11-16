use ::arch::defs::XEN_LEGACY_MAX_VCPUS;

pub mod event;
pub mod sched;

#[macro_use]
pub mod console;

pub mod hypercall;

const MAX_GUEST_CMDLINE: usize = 1024;

extern {
    pub static _shared_info: SharedInfo;
}

#[repr(C)]
pub struct DomUConsole {
    pub mfn: ::arch::defs::XenPfn,
    pub evtchn: u32,
}

#[repr(C)]
pub struct StartInfo {
    pub magic: [u8; 32],
    pub nr_pages: ::arch::defs::Ulong,
    pub shared_info: ::arch::defs::Ulong,
    pub flags: u32,
    pub store_mfn: ::arch::defs::XenPfn,
    pub store_evtchn: u32,
    pub domu_console: DomUConsole,
    pub pt_base: ::arch::defs::Ulong,
    pub nr_pt_frames: ::arch::defs::Ulong,
    pub mfn_list: ::arch::defs::Ulong,
    pub mod_start: ::arch::defs::Ulong,
    pub mod_len: ::arch::defs::Ulong,
    pub cmd_line: [u8; MAX_GUEST_CMDLINE],
    pub first_p2m_pfn: ::arch::defs::Ulong,
    pub nr_p2m_frames: ::arch::defs::Ulong,
}

#[repr(C)]
pub struct VcpuTimeInfo {
    pub version: u32,
    pub pad0: u32,
    pub tsc_timestamp: u64,
    pub system_time: u64,
    pub tsc_to_system_mul: u32,
    pub tsc_shift: i8,
    pub pad: [i8; 3],
}

#[repr(C)]
pub struct VcpuInfo{
    pub evtchn_upcall_pending: u8,
    pub evtchn_upcall_mask : u8,
    pub pad0 : u8,
    pub evtchn_pending_sel: ::arch::defs::XenUlong,
    pub arch: ::arch::defs::ArchVcpuInfo,
    pub time: VcpuTimeInfo,
}

#[repr(C)]
pub struct SharedInfo {
    pub vcpu_info: [VcpuInfo; XEN_LEGACY_MAX_VCPUS],
    pub evtchn_pending: [::arch::defs::XenUlong; ::arch::defs::XENULONGSIZE * 8],
    pub evtchn_mask: [::arch::defs::XenUlong; ::arch::defs::XENULONGSIZE * 8],
    pub wc_version: u32,
    pub wc_sec: u32,
    pub wc_nsec: u32,
    pub arch: ::arch::defs::ArchSharedInfo,
}
