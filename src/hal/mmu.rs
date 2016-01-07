//! Generic definitions related to the MMU

use core::ops::{Add, AddAssign, Deref};

use hal::arch::mmu::PageEntry;
use hal::arch::mmu::{
    PAGE_SHIFT,
    OFFSET_MASK,
    L1_PAGE_SHIFT,
    L2_PAGE_SHIFT,
    L3_PAGE_SHIFT,
    L4_PAGE_SHIFT
};

pub trait Address {
    type Repr;
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[cfg(feature = "xen")]
/// A physical address
pub struct Paddr(u64);

#[cfg(feature = "xen")]
impl Address for Paddr { type Repr = u64; }

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[cfg(feature = "xen")]
/// A machine address
pub struct Maddr(u64);

#[cfg(feature = "xen")]
impl Address for Maddr { type Repr = u64; }

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[cfg(not(feature = "xen"))]
/// A physical address
pub struct Paddr(usize);

#[cfg(not(feature = "xen"))]
impl Address for Paddr { type Repr = usize; }

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[cfg(not(feature = "xen"))]
/// A machine address
pub struct Maddr(usize);

#[cfg(not(feature = "xen"))]
impl Address for Maddr { type Repr = usize; }

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
/// A virtual address
pub struct Vaddr(usize);

impl Address for Vaddr { type Repr = usize; }

#[repr(C)]
#[derive(Clone, Copy)]
/// Physical frame number
pub struct Pfn(usize);

#[repr(C)]
#[derive(Clone, Copy)]
/// Machine frame number
pub struct Mfn(usize);

impl Vaddr {
    /// Create a new virtual address
    pub fn new(value: <Vaddr as Address>::Repr) -> Self {
        Vaddr(value)
    }

    /// Create a new virtual address from a constant pointer
    pub fn from_ptr<T>(ptr: *const T) -> Self {
        Vaddr(ptr as usize)
    }

    /// Create a new virtual address from a mutable pointer
    pub fn from_mut_ptr<T>(ptr: *mut T) -> Self {
        Vaddr(ptr as usize)
    }

    /// Get the offset in the 4th layer of MMU configuration tables
    ///
    /// Note: This value will be garbage if the table does not exist
    pub fn l4_offset(&self) -> usize {
        ((self.0 >> L4_PAGE_SHIFT) & OFFSET_MASK) as usize
    }

    /// Get the offset in the 3rd layer of MMU configuration tables
    ///
    /// Note: This value will be garbage if the table does not exist
    pub fn l3_offset(&self) -> usize {
        ((self.0 >> L3_PAGE_SHIFT) & OFFSET_MASK) as usize
    }

    /// Get the offset in the 2nd layer of MMU configuration tables
    pub fn l2_offset(&self) -> usize {
        ((self.0 >> L2_PAGE_SHIFT) & OFFSET_MASK) as usize
    }

    /// Get the offset in the 1st layer of MMU configuration tables
    pub fn l1_offset(&self) -> usize {
        ((self.0 >> L1_PAGE_SHIFT) & OFFSET_MASK) as usize
    }

    /// Get the internal value as a constant rust pointer
    ///
    /// This is unsafe as the internal value might not be a valid pointer
    pub unsafe fn as_ptr<T>(&self) -> *const T {
        self.0 as *const u8 as *const T
    }

    /// Get the internal value as a mutable rust pointer
    ///
    /// This is unsafe as the internal value might not be a valid pointer
    pub unsafe fn as_mut_ptr<T>(&mut self) -> *mut T {
        self.0 as *mut u8 as *mut T
    }

    /// Increment the virtual address
    ///
    /// `count` is the number of bytes you want to increment the pointed value
    pub unsafe fn incr(mut self, count: usize) -> Self {
        self.0 += count as <Self as Address>::Repr;
        self
    }
}

impl Paddr {
    /// Create a new physical address
    pub fn new(value: <Paddr as Address>::Repr) -> Self {
        Paddr(value)
    }

    /// Increment the virtual address
    ///
    /// `count` is the number of bytes you want to increment the pointed value
    pub unsafe fn incr(mut self, count: usize) -> Self {
        self.0 += count as <Self as Address>::Repr;
        self
    }
}

impl Maddr {
    /// Create a new machine address
    pub fn new(value: <Maddr as Address>::Repr) -> Self {
        Maddr(value)
    }

    /// Increment the virtual address
    ///
    /// `count` is the number of bytes you want to increment the pointed value
    pub unsafe fn incr(mut self, count: usize) -> Self {
        self.0 += count as <Self as Address>::Repr;
        self
    }
}

impl Pfn {
    /// Create a new physical frame number
    pub fn new(value: usize) -> Self {
        Pfn(value)
    }
}

impl Mfn {
    /// Create a new machine frame number
    pub fn new(value: usize) -> Self {
        Mfn(value)
    }
}

impl Add<usize> for Pfn {
    type Output = Pfn;

    fn add(mut self, rhs: usize) -> Self::Output {
        self.0 += rhs;
        self
    }
}
impl AddAssign<usize> for Pfn {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Add<usize> for Mfn {
    type Output = Mfn;

    fn add(mut self, rhs: usize) -> Self::Output {
        self.0 += rhs;
        self
    }
}
impl AddAssign<usize> for Mfn {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Deref for Vaddr {
    type Target = <Self as Address>::Repr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Paddr {
    type Target = <Self as Address>::Repr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Maddr {
    type Target = <Self as Address>::Repr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Pfn {
    type Target = usize;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Mfn {
    type Target = usize;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Pfn> for Vaddr {
    fn from(pfn: Pfn) -> Vaddr {
        Vaddr(pfn.0 << PAGE_SHIFT)
    }
}

impl From<Vaddr> for Pfn {
    fn from(vaddr: Vaddr) -> Pfn {
        Pfn(vaddr.0 >> PAGE_SHIFT)
    }
}

impl From<Mfn> for Vaddr {
    fn from(mfn: Mfn) -> Vaddr {
        Vaddr::from(Pfn::from(mfn))
    }
}

impl From<Mfn> for Maddr {
    fn from(mfn: Mfn) -> Maddr {
        Maddr((mfn.0 as <Self as Address>::Repr) << PAGE_SHIFT)
    }
}

impl From<Vaddr> for Mfn {
    fn from(vaddr: Vaddr) -> Mfn {
        Mfn::from(Pfn::from(vaddr))
    }
}

impl From<PageEntry> for Vaddr {
    fn from(entry: PageEntry) -> Vaddr {
        Vaddr::from(Mfn::from(entry))
    }
}

#[cfg(feature = "xen")]
impl From<PageEntry> for Mfn {
    fn from(entry: PageEntry) -> Mfn {
        use hal::arch::mmu::PTE_MASK;

        let page_entry_value = entry.mask(PTE_MASK).value() >> PAGE_SHIFT;

        Mfn(PageEntry::new(page_entry_value).value() as usize)
    }
}

#[cfg(feature = "xen")]
impl From<Maddr> for PageEntry {
    fn from(maddr: Maddr) -> PageEntry {
        use hal::arch::mmu::PTE_FLAGS_MASK;
        use hal::arch::mmu::PageFlags;

        let value = (maddr.0 as u64) & !PTE_FLAGS_MASK;

        PageEntry::new(value).set(PageFlags::Present).set(PageFlags::Writable)
    }
}

#[cfg(feature = "xen")]
impl From<Mfn> for PageEntry {
    fn from(mfn: Mfn) -> PageEntry {
        PageEntry::from(Maddr::from(mfn))
    }
}

#[cfg(feature = "xen")]
impl From<Pfn> for PageEntry {
    fn from(pfn: Pfn) -> PageEntry {
        PageEntry::from(Mfn::from(pfn))
    }
}

#[cfg(feature = "xen")]
impl From<Mfn> for Pfn {
    fn from(mfn: Mfn) -> Pfn {
        use hal::xen::defs::MACH2PHYS_VIRT_START;

        let mtp_mapping: *const Pfn = MACH2PHYS_VIRT_START as *const Pfn;

        unsafe {
            *mtp_mapping.offset(mfn.0 as isize)
        }
    }
}

#[cfg(feature = "xen")]
impl From<Pfn> for Mfn {
    fn from(pfn: Pfn) -> Mfn {
        // XXX: Won't work for ARM
        use hal::xen::boot::start_info;

        unsafe {
            let ptm_mapping: *const Mfn = (*start_info).mfn_list as *const Mfn;

            *ptm_mapping.offset(pfn.0 as isize)
        }
    }
}
