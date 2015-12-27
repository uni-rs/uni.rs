//! Definition of various functions, constants and types useful only for
//! x86 architectures

use core::ops::{BitAnd, BitOr};

pub const PAGE_SHIFT: usize = 12;

/// Size of a page
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;

/// Mask to apply to a page table entry to isolate the flags
pub const PTE_FLAGS_MASK: u64 = (PAGE_SIZE - 1) as u64;

// Special case: Xen uses PAE for x86 which changes some constants related to
// paging. We are in the arch part of the hal, and this is Xen related,
// but it would not make much sense to define so constants for every single
// platform because of one 'exception'

#[cfg(target_arch = "x86_64")]
pub const PML4_OFFSET_SHIFT: usize = 39;

#[cfg(target_arch = "x86")]
pub const PML4_OFFSET_SHIFT: usize = 0;

pub const PD_OFFSET_SHIFT: usize = 30;

#[cfg(any(feature = "xen", target_arch = "x86_64"))]
pub const PT_OFFSET_SHIFT: usize = 21;

#[cfg(all(not(feature = "xen"), target_arch = "x86"))]
pub const PT_OFFSET_SHIFT: usize = 22;

pub const PDP_OFFSET_SHIFT: usize = 12;

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

pub unsafe fn atomic_set_bit<T>(nr: usize, addr: *mut T) {
    asm!("lock bts $1, $0"
         : "=*m" (addr as *mut u32)
         : "Ir" (nr)
         : "memory"
         : "volatile");
}

pub unsafe fn atomic_clear_bit<T>(nr: usize, addr: *mut T) {
    asm!("lock btr $1, $0"
         : "=*m" (addr as *mut u32)
         : "Ir" (nr)
         : "memory"
         : "volatile");
}

pub fn first_bit(data: usize) -> usize {
    unsafe {
        let ret;

        asm!("bsf $1, $0\n\t\
              jnz 1f\n\t
              mov $$0, $0\n\t
              1:\n\t"
             : "=r" (ret)
             : "r" (data)
             :: "volatile");

        ret
    }
}

pub fn wmb() {
    unsafe {
        asm!("sfence" ::: "memory" : "volatile");
    }
}

#[test]
pub fn test_set_and_clear() {
    let mut array = [0u32; 4];

    unsafe {
        atomic_set_bit(1, &mut array[0] as *mut u32);
        atomic_set_bit(2, &mut array[0] as *mut u32);
        atomic_set_bit(3, &mut array[0] as *mut u32);
        atomic_set_bit(32, &mut array[0] as *mut u32);
        atomic_set_bit(33, &mut array[0] as *mut u32);
        atomic_set_bit(34, &mut array[0] as *mut u32);
        atomic_set_bit(127, &mut array[0] as *mut u32);

        assert_eq!(array[0], 0xE);
        assert_eq!(array[1], 0x7);
        assert_eq!(array[2], 0x0);
        assert_eq!(array[3], 0x80000000);

        atomic_clear_bit(1, &mut array[0] as *mut u32);
        atomic_clear_bit(2, &mut array[0] as *mut u32);
        atomic_clear_bit(3, &mut array[0] as *mut u32);
        atomic_clear_bit(32, &mut array[0] as *mut u32);
        atomic_clear_bit(33, &mut array[0] as *mut u32);
        atomic_clear_bit(34, &mut array[0] as *mut u32);
        atomic_clear_bit(127, &mut array[0] as *mut u32);

        assert_eq!(array[0], 0x0);
        assert_eq!(array[1], 0x0);
        assert_eq!(array[2], 0x0);
        assert_eq!(array[3], 0x0);
    }
}

#[test]
pub fn test_first_bit() {
    assert_eq!(first_bit(0x0), 0);
    assert_eq!(first_bit(0x1), 0);
    assert_eq!(first_bit(0x2), 1);
    assert_eq!(first_bit(0x3), 0);
    assert_eq!(first_bit(0x8000), 15);
    assert_eq!(first_bit(0xFF80), 7);
    assert_eq!(first_bit(0x80000000), 31);
}
