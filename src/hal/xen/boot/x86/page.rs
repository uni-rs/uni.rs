//! Page index and address manipulation

use hal::x86::{
    PageEntry,

    L1_PAGE_SHIFT,
    L2_PAGE_SHIFT,
    L3_PAGE_SHIFT,
    L4_PAGE_SHIFT,
    PAGE_SHIFT,
    OFFSET_MASK,
    PTE_MASK
};

use hal::xen::defs::MACH2PHYS_VIRT_START;

use super::start_info;

macro_rules! pte {
    ($x:expr) => {{
        use $crate::hal::x86::PTE_FLAGS_MASK;
        use $crate::hal::x86::{PageEntry, PageFlags};

        PageEntry::new($x as u64 & !PTE_FLAGS_MASK)
            .set(PageFlags::Present)
            .set(PageFlags::Writable)
    }}
}

pub type Vaddr = usize;
pub type Maddr = u64;
pub type Pfn = usize;
pub type Mfn = usize;

pub fn mfn_to_pfn(mfn: Mfn) -> Pfn {
    unsafe {
        let mtp_mapping: *const Pfn = MACH2PHYS_VIRT_START as *const Pfn;

        *mtp_mapping.offset(mfn as isize)
    }
}

pub fn pfn_to_mfn(pfn: Pfn) -> Mfn {
    unsafe {
        let ptm_mapping: *const Mfn = (*start_info).mfn_list as *const Mfn;

        *ptm_mapping.offset(pfn as isize)
    }
}

pub fn pfn_to_vaddr(pfn: Pfn) -> Vaddr {
    pfn << PAGE_SHIFT
}

pub fn vaddr_to_pfn(vaddr: Vaddr) -> Pfn {
    vaddr >> PAGE_SHIFT
}

pub fn mfn_to_vaddr(mfn: Mfn) -> Vaddr {
    pfn_to_vaddr(mfn_to_pfn(mfn))
}

pub fn vaddr_to_mfn(vaddr: Vaddr) -> Mfn {
    pfn_to_mfn(vaddr_to_pfn(vaddr))
}

pub fn pte_to_vaddr(entry: PageEntry) -> Vaddr {
    mfn_to_vaddr(pte_to_mfn(entry))
}

pub fn pte_to_mfn(entry: PageEntry) -> Mfn {
    PageEntry::new(entry.mask(PTE_MASK).value() >> PAGE_SHIFT).value() as Mfn
}

pub fn mfn_to_pte(mfn: Mfn) -> PageEntry {
    pte!((mfn as u64) << PAGE_SHIFT)
}

pub fn pfn_to_pte(pfn: Pfn) -> PageEntry {
    mfn_to_pte(pfn_to_mfn(pfn))
}

pub fn pml4_offset(vaddr: Vaddr) -> usize {
    ((vaddr >> L4_PAGE_SHIFT) & OFFSET_MASK) as usize
}

pub fn pdp_offset(vaddr: Vaddr) -> usize {
    ((vaddr >> L3_PAGE_SHIFT) & OFFSET_MASK) as usize
}

pub fn pd_offset(vaddr: Vaddr) -> usize {
    ((vaddr >> L2_PAGE_SHIFT) & OFFSET_MASK) as usize
}

pub fn pt_offset(vaddr: Vaddr) -> usize {
    ((vaddr >> L1_PAGE_SHIFT) & OFFSET_MASK) as usize
}
