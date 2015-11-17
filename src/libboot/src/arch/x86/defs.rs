//! Definition of types and constants related to the x86 architecture

pub type PageTableEntry = u64;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;

pub const PAGE_WRITABLE: u64 = 2;
pub const PAGE_PRESENT: u64 = 1;

// Page table entry flags
pub const PTE_MASK: u64 = (1 << 44) - 1;
pub const PTE_FLAGS: u64 = PAGE_WRITABLE | PAGE_PRESENT;
pub const PTE_FLAGS_MASK: u64 = (PAGE_SIZE - 1) as u64;

pub const OFFSET_MASK: usize = 0x1FF;

#[cfg(target_arch = "x86_64")]
pub const PML4_OFFSET_SHIFT: usize = 39;

#[cfg(target_arch = "x86")]
pub const PML4_OFFSET_SHIFT: usize = 0;

pub const PDP_OFFSET_SHIFT: usize = 30;
pub const PD_OFFSET_SHIFT: usize = 21;
pub const PT_OFFSET_SHIFT: usize = 12;

pub const PTE_PER_TABLE: usize = 512;
