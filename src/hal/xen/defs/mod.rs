mod x86;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use self::x86::*;

pub const DOMID_SELF: u16 = 0x7FF0;

#[repr(C)]
pub struct DomUConsole {
    pub mfn: Pfn,
    pub evtchn: u32,
}

pub const MAX_GUEST_CMDLINE: usize = 1024;

#[repr(C)]
pub struct StartInfo {
    pub magic: [u8; 32],
    pub nr_pages: Ulong,
    pub shared_info: Ulong,
    pub flags: u32,
    pub store_mfn: Pfn,
    pub store_evtchn: u32,
    pub domu_console: DomUConsole,
    pub pt_base: Ulong,
    pub nr_pt_frames: Ulong,
    pub mfn_list: Ulong,
    pub mod_start: Ulong,
    pub mod_len: Ulong,
    pub cmd_line: [u8; MAX_GUEST_CMDLINE],
    pub first_p2m_pfn: Ulong,
    pub nr_p2m_frames: Ulong,
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
    pub evtchn_pending_sel: Ulong,
    pub arch: ArchVcpuInfo,
    pub time: VcpuTimeInfo,
}

#[repr(C)]
pub struct SharedInfo {
    pub vcpu_info: [VcpuInfo; XEN_LEGACY_MAX_VCPUS],
    pub evtchn_pending: [Ulong; ULONG_SIZE * 8],
    pub evtchn_mask: [Ulong; ULONG_SIZE * 8],
    pub wc_version: u32,
    pub wc_sec: u32,
    pub wc_nsec: u32,
    pub arch: ArchSharedInfo,
}

pub type EvtchnPort = u32;

// XXX: Console related stuff should live in hal::xen::console
pub type ConsRingIdx = u32;

#[repr(C)]
pub struct ConsoleInterface {
    pub input: [u8; 1024],
    pub output: [u8; 2048],
    pub in_cons: ConsRingIdx,
    pub in_prod: ConsRingIdx,
    pub out_cons: ConsRingIdx,
    pub out_prod: ConsRingIdx,
}

// XXX: Xenstore related stuff should live in hal::xen::store
pub const XENSTORE_RING_SIZE: usize = 1024;

pub type XenstoreRingIdx = u32;

#[repr(C)]
pub struct XenstoreInterface {
    pub req: [u8; XENSTORE_RING_SIZE],
    pub rsp: [u8; XENSTORE_RING_SIZE],
    pub req_cons: XenstoreRingIdx,
    pub req_prod: XenstoreRingIdx,
    pub rsp_cons: XenstoreRingIdx,
    pub rsp_prod: XenstoreRingIdx,
}
