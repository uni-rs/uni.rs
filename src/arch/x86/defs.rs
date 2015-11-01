//! Definitions for the x86 architecture

use arch::x86_common::memory::page::Vaddr;

pub use arch::x86_common::defs::*;

pub const MACH2PHYS_VIRT_START: Vaddr = 0xF5800000;

pub const XENULONGSIZE: usize = 4;

pub const HYPERVISOR_START: Vaddr = 0xF5800000;

pub const PML4_OFFSET_SHIFT: usize = 0;
