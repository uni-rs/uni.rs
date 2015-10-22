//! Definitions for the x86_64 architecture

pub use ::arch::x86_common::defs::*;

pub const MACH2PHYS_VIRT_START: Ulong = 0xFFFF800000000000;

pub const XENULONGSIZE: usize = 8;
