#[cfg(target_arch = "x86")]
mod defs {
    pub const ULONG_SIZE: usize = 4;
    pub const MACH2PHYS_VIRT_START: usize = 0xF5800000;
    pub const FLAT_KERNEL_CS: u16 = 0xe019;
}

#[cfg(target_arch = "x86_64")]
mod defs {
    pub const ULONG_SIZE: usize = 8;
    pub const MACH2PHYS_VIRT_START: usize = 0xFFFF800000000000;
    pub const FLAT_KERNEL_CS: u16 = 0xe033;
}

pub use self::defs::*;

pub const XEN_LEGACY_MAX_VCPUS: usize = 32;

/// unsigned long
pub type Ulong = usize;

/// xen_pfn_t
pub type Pfn = Ulong;

/// struct arch_vcpu_info
#[repr(C)]
#[cfg(target_arch = "x86")]
pub struct ArchVcpuInfo {
    pub cr2: Ulong,
    pub pad: [Ulong; 5],
}

/// struct arch_vcpu_info
#[repr(C)]
#[cfg(target_arch = "x86_64")]
pub struct ArchVcpuInfo {
    pub cr2: Ulong,
    pub pad: Ulong,
}

/// struct arch_shared_info
#[repr(C)]
pub struct ArchSharedInfo {
    pub max_pfn: Ulong,
    pub pfn_to_mfn_frame_list_list: Ulong,
    pub nmi_reason: Ulong,
}
