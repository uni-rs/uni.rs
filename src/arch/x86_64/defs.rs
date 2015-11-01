//! Definitions for the x86_64 architecture

use arch::x86_common::memory::page::Vaddr;

pub use arch::x86_common::defs::*;

pub const MACH2PHYS_VIRT_START: Vaddr = 0xFFFF800000000000;

pub const XENULONGSIZE: usize = 8;

pub const HYPERVISOR_START: Vaddr = 0xFFFF800000000000;

pub const PML4_OFFSET_SHIFT: usize = 39;
