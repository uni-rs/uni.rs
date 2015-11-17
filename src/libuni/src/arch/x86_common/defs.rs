//! Common definition for the x86_* platform

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
