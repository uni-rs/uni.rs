//! Page index and address manipulation

use hal::x86::PageEntry;
use hal::x86::PAGE_SHIFT;

use hal::xen::defs::MACH2PHYS_VIRT_START;

use super::start_info;


use super::defs::{OFFSET_MASK, PTE_MASK};


macro_rules! pte {
    ($x:expr) => {{
        use $crate::hal::x86::PageFlags;

        use $crate::hal::x86::PageEntry;
        use $crate::hal::xen::boot::arch::defs::PTE_FLAGS_MASK;

        PageEntry::<u64>::new($x as u64 & !PTE_FLAGS_MASK)
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

pub fn pte_to_vaddr(entry: PageEntry<u64>) -> Vaddr {
    mfn_to_vaddr(pte_to_mfn(entry))
}

pub fn pte_to_mfn(entry: PageEntry<u64>) -> Mfn {
    PageEntry::new(entry.mask(PTE_MASK).value() >> PAGE_SHIFT).value() as Mfn
}

pub fn mfn_to_pte(mfn: Mfn) -> PageEntry<u64> {
    pte!((mfn as u64) << PAGE_SHIFT)
}

pub fn pfn_to_pte(pfn: Pfn) -> PageEntry<u64> {
    mfn_to_pte(pfn_to_mfn(pfn))
}

pub fn pml4_offset(vaddr: Vaddr) -> usize {
    ((vaddr >> ::hal::xen::boot::arch::defs::PML4_OFFSET_SHIFT) & OFFSET_MASK) as usize
}

pub fn pdp_offset(vaddr: Vaddr) -> usize {
    ((vaddr >> ::hal::xen::boot::arch::defs::PDP_OFFSET_SHIFT) & OFFSET_MASK) as usize
}

pub fn pd_offset(vaddr: Vaddr) -> usize {
    ((vaddr >> ::hal::xen::boot::arch::defs::PD_OFFSET_SHIFT) & OFFSET_MASK) as usize
}

pub fn pt_offset(vaddr: Vaddr) -> usize {
    ((vaddr >> ::hal::xen::boot::arch::defs::PT_OFFSET_SHIFT) & OFFSET_MASK) as usize
}
