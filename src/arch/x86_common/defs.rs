//! Common definition for the x86_* platform

use core::usize::MAX as MAX_USIZE;

// unsigned long
pub type Ulong = usize;

// xen_pfn_t
pub type XenPfn = Ulong;

// xen_ulong_t
pub type XenUlong = Ulong;

// struct arch_vcpu_info
#[repr(C)]
pub struct ArchVcpuInfo {
    pub cr2: Ulong,
    pub pad: [Ulong; 5],
}

// struct arch_shared_info
#[repr(C)]
pub struct ArchSharedInfo {
    pub max_pfn: Ulong,
    pub pfn_to_mfn_frame_list_list: Ulong,
    pub nmi_reason: Ulong,
}

pub const MAX_ULONG: Ulong = MAX_USIZE;
pub const XEN_LEGACY_MAX_VCPUS: usize = 32;
