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

pub type TableEntry = u64;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;

pub const PAGE_WRITABLE: u64 = 2;
pub const PAGE_PRESENT: u64 = 1;

// Page table entry flags
pub const PAGE_FLAGS: u64 = PAGE_WRITABLE | PAGE_PRESENT;

pub const PTE_FLAGS_MASK: u64 = (PAGE_SIZE - 1) as u64;

pub const PTE_MASK: u64 = (1 << 44) - 1;

pub const PDP_OFFSET_SHIFT: usize = 30;
pub const PD_OFFSET_SHIFT: usize = 21;
pub const PT_OFFSET_SHIFT: usize = 12;

pub const OFFSET_MASK: usize = 0x1FF;

pub const PTE_PER_TABLE: usize = 512;
