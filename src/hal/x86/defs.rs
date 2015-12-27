//! Generic architecture definitions

use hal::arch::mmu::PAGE_SHIFT;

/// Size of a page
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
