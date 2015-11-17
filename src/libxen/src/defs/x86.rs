#[cfg(target_arch = "x86")]
pub const ULONG_SIZE: usize = 4;

#[cfg(target_arch = "x86_64")]
pub const ULONG_SIZE: usize = 8;

#[cfg(target_arch = "x86")]
pub const MACH2PHYS_VIRT_START: usize = 0xF5800000;

#[cfg(target_arch = "x86_64")]
pub const MACH2PHYS_VIRT_START: usize = 0xFFFF800000000000;

pub const XEN_LEGACY_MAX_VCPUS: usize = 32;

/// unsigned long
pub type Ulong = usize;

/// xen_pfn_t
pub type Pfn = Ulong;

/// struct arch_vcpu_info
#[repr(C)]
pub struct ArchVcpuInfo {
    pub cr2: Ulong,
    pub pad: [Ulong; 5],
}

/// struct arch_shared_info
#[repr(C)]
pub struct ArchSharedInfo {
    pub max_pfn: Ulong,
    pub pfn_to_mfn_frame_list_list: Ulong,
    pub nmi_reason: Ulong,
}
