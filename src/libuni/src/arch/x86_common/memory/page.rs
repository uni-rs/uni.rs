//! Page index and address manipulation

use arch::defs::TableEntry;

use arch::x86_common::start_info;

use arch::defs::OFFSET_MASK;
use arch::defs::PTE_MASK;
use arch::defs::PAGE_SHIFT;
use arch::defs::MACH2PHYS_VIRT_START;

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

pub fn pte_to_vaddr(entry: TableEntry) -> Vaddr {
    mfn_to_vaddr(pte_to_mfn(entry))
}

pub fn pte_to_mfn(entry: TableEntry) -> Mfn {
    ((entry & PTE_MASK) >> PAGE_SHIFT as TableEntry) as Mfn
}

pub fn mfn_to_pte(mfn: Mfn) -> TableEntry {
    pte!((mfn as TableEntry) << PAGE_SHIFT)
}

pub fn pfn_to_pte(pfn: Pfn) -> TableEntry {
    mfn_to_pte(pfn_to_mfn(pfn))
}

pub fn pml4_offset(vaddr: Vaddr) -> usize {
    ((vaddr >> ::arch::defs::PML4_OFFSET_SHIFT) & OFFSET_MASK) as usize
}

pub fn pdp_offset(vaddr: Vaddr) -> usize {
    ((vaddr >> ::arch::defs::PDP_OFFSET_SHIFT) & OFFSET_MASK) as usize
}

pub fn pd_offset(vaddr: Vaddr) -> usize {
    ((vaddr >> ::arch::defs::PD_OFFSET_SHIFT) & OFFSET_MASK) as usize
}

pub fn pt_offset(vaddr: Vaddr) -> usize {
    ((vaddr >> ::arch::defs::PT_OFFSET_SHIFT) & OFFSET_MASK) as usize
}
