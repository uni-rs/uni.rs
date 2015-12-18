//! Definition of types and constants related to the x86 architecture

use hal::x86::PAGE_SIZE;

pub type PageTableEntry = u64;

// Page table entry flags
pub const PTE_MASK: u64 = (1 << 44) - 1;
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
