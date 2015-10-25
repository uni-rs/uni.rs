pub mod hypercall;

const MAX_GUEST_CMDLINE: usize = 1024;

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
