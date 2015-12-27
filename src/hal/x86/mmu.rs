//! Architecture dependent MMU definitions

use core::ops::{BitAnd, BitOr};

use hal::arch::defs::PAGE_SIZE;

pub const PAGE_SHIFT: usize = 12;

/// Mask to apply to a page table entry to isolate the flags
pub const PTE_FLAGS_MASK: u64 = (PAGE_SIZE - 1) as u64;

// Special case: Xen uses PAE for x86 which changes some constants related to
// paging. We are in the arch part of the hal, and this is Xen related,
// but it would not make much sense to define so constants for every single
// platform because of one 'exception'

#[cfg(target_arch = "x86_64")]
pub const L4_PAGE_SHIFT: usize = 39;

#[cfg(target_arch = "x86")]
pub const L4_PAGE_SHIFT: usize = 0;

pub const L3_PAGE_SHIFT: usize = 30;

#[cfg(any(feature = "xen", target_arch = "x86_64"))]
pub const L2_PAGE_SHIFT: usize = 21;

#[cfg(all(not(feature = "xen"), target_arch = "x86"))]
pub const L2_PAGE_SHIFT: usize = 22;

pub const L1_PAGE_SHIFT: usize = 12;

#[cfg(any(feature = "xen", target_arch = "x86_64"))]
pub const PTE_PER_TABLE: usize = 512;

#[cfg(all(not(feature = "xen"), target_arch = "x86"))]
pub const PTE_PER_TABLE: usize = 1024;

#[cfg(any(feature = "xen", target_arch = "x86_64"))]
pub const PTE_MASK: u64 = (1 << 44) - 1;

#[cfg(any(feature = "xen", target_arch = "x86_64"))]
pub const OFFSET_MASK: usize = 0x1FF;

#[cfg(all(not(feature = "xen"), target_arch = "x86"))]
pub const OFFSET_MASK: usize = 0x3FF;
#[cfg(feature = "xen")]
pub type PageEntry = PageEntryImp<u64>;

#[cfg(not(feature = "xen"))]
pub type PageEntry = PageEntryImp;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PageEntryImp<T = usize> {
    entry: T,
}

// XXX: Is there a better solution than that ?
impl<T> PageEntryImp<T> where T: BitOr<T> + From<<T as BitOr>::Output>,
                              T: BitAnd<T> + From<<T as BitAnd>::Output>,
                              T: PartialEq<T> + From<PageFlags> + Copy {
    #[inline]
    pub fn new(entry: T) -> Self {
        PageEntryImp {
            entry: entry
        }
    }

    #[inline]
    pub fn value(&self) -> T {
        self.entry
    }

    #[inline]
    pub fn mask(mut self, mask: T) -> Self {
        self.entry = T::from(self.entry & mask);
        self
    }

    #[inline]
    pub fn set(mut self, flag: PageFlags) -> Self {
        self.entry = T::from(self.entry | T::from(flag));
        self
    }

    #[inline]
    pub fn has(&self, flags: PageFlags) -> bool {
        let flags = T::from(flags);

        T::from(self.entry & flags) == flags
    }
}

#[allow(dead_code)]
pub enum PageFlags {
    Present = 0x1,
    Writable = 0x2,
    User = 0x4,
    WriteThrough = 0x8,
    CacheDisabled = 0x10,
}

impl From<PageFlags> for usize {
    #[inline]
    fn from(flags: PageFlags) -> Self {
        flags as usize
    }
}

impl From<PageFlags> for u32 {
    #[inline]
    fn from(flags: PageFlags) -> Self {
        flags as u32
    }
}

impl From<PageFlags> for u64 {
    #[inline]
    fn from(flags: PageFlags) -> Self {
        flags as u64
    }
}
