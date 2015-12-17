use core::ptr;
use core::mem::size_of;

use hal::xen::boot::libc::memset;

use super::page;

use hal::xen::memory::{MmuUpdate, MapFlags};
use hal::xen::memory::{mmu_update, update_va_mapping};

use super::defs::PageTableEntry;

use super::defs::PAGE_SIZE;
use super::defs::PAGE_SHIFT;
use super::defs::PAGE_PRESENT;
use super::defs::PTE_PER_TABLE;

macro_rules! div_up {
    ($x:expr, $y:expr) => {
        ($x - 1) / $y + 1 + (if $x % $y == 0 { 1 } else { 0 })
    }
}

const MAX_UPDATES: usize = 100;

/// Helper that creates an identity mapping between physical memory and virtual
/// memory.
pub struct IdentityMapper {
    top_level_table: *const PageTableEntry,
    admin_pfn_pool: page::Pfn,
    reg_pfn_pool: page::Pfn,
    updates: [MmuUpdate; 100],
    nr_update: usize,
    pub area_start: page::Vaddr,
    pub area_end: page::Vaddr,
}

impl IdentityMapper {
    pub fn new(pt_base: page::Vaddr, nr_pt_frames: usize,
               nr_pages: usize) -> IdentityMapper {
        let mut admin_page_count;

        // We need 1 page table every PTE_PER_TABLE pages
        admin_page_count = div_up!(nr_pages, PTE_PER_TABLE);

        // We need 1 page directory every PTE_PER_TABLE ^ 2 pages
        admin_page_count += div_up!(nr_pages, PTE_PER_TABLE * PTE_PER_TABLE);

        // We need 1 page directory pointer table every PTE_PER_TABLE ** 3
        // pages
        admin_page_count += div_up!(nr_pages, PTE_PER_TABLE * PTE_PER_TABLE *
                                              PTE_PER_TABLE);

        if cfg!(target_arch = "x86_64") {
            // We need 1 page map level 4.
            admin_page_count += 1;
        }

        // We already have some administration pages allocated by xen
        if admin_page_count < nr_pt_frames {
            admin_page_count = 0;
        } else {
            admin_page_count -= nr_pt_frames;
        }

        let pfn_base = page::vaddr_to_pfn(pt_base) + 1;
        let reg_pfn_base = pfn_base + nr_pt_frames + admin_page_count;
        let nr_page_to_map = nr_pages - reg_pfn_base;

        IdentityMapper {
            top_level_table: pt_base as *const PageTableEntry,
            admin_pfn_pool: pfn_base + nr_pt_frames,
            reg_pfn_pool: reg_pfn_base,
            updates: [MmuUpdate::null(); 100],
            nr_update: 0,
            area_start: page::pfn_to_vaddr(reg_pfn_base),
            area_end: page::pfn_to_vaddr(reg_pfn_base) + nr_page_to_map *
                                                         PAGE_SIZE,
        }
    }

    // Xen will prevent a page from being a page table (or higher level
    // table) if this page is mapped somewhere as writable or if it has
    // some non valid entries. This is why we first zero the page, then
    // map it as read only
    unsafe fn add_admin_page(&mut self, table: *const PageTableEntry,
                             offset: usize) {
        let new_entry_pfn = self.admin_pfn_pool;
        let new_entry_mfn = page::pfn_to_mfn(new_entry_pfn);
        let new_entry_vaddr = page::pfn_to_vaddr(new_entry_pfn);
        let new_entry_pte = page::pfn_to_pte(new_entry_pfn);

        // We map this new page in our virtual address space so that we
        // can clear it
        update_va_mapping(new_entry_vaddr, new_entry_pte,
                          MapFlags::InvlpgLocal);

        // Clear the page before mapping it
        memset(new_entry_vaddr as *mut u8, 0, PAGE_SIZE);

        // We remap it as read only
        update_va_mapping(new_entry_vaddr,
                          ((new_entry_mfn as page::Maddr) << PAGE_SHIFT) |
                          PAGE_PRESENT, MapFlags::InvlpgLocal);

        let table_mfn = page::vaddr_to_mfn(table as page::Vaddr);
        let mut table_mach : page::Maddr;

        table_mach = (table_mfn as page::Maddr) << PAGE_SHIFT;
        table_mach += (offset * size_of::<PageTableEntry>()) as page::Maddr;

        // We add a new page to the table
        self.add_mmu_update(table_mach, new_entry_pte, true);

        self.admin_pfn_pool += 1;
    }

    unsafe fn add_mmu_update(&mut self, ptr: page::Maddr, val: PageTableEntry,
                      force_update: bool) {
        self.updates[self.nr_update as usize] = MmuUpdate::new(ptr, val);

        self.nr_update += 1;

        if force_update || self.nr_update == MAX_UPDATES {
            self.flush_updates()
        }
    }

    unsafe fn flush_updates(&mut self) {
        if self.nr_update == 0 {
            return;
        }

        let updates_ptr = &self.updates[0] as *const MmuUpdate;
        let ret: i32;

        ret = mmu_update(updates_ptr, self.nr_update, ptr::null_mut());

        if ret < 0 {
            panic!("Mmu update failed with err = {}", ret);
        }

        self.nr_update = 0;
    }

    unsafe fn extract_entry(&mut self, table: *const PageTableEntry,
                            offset: usize) -> *const PageTableEntry {
        let mut entry = *table.offset(offset as isize);

        if (entry & PAGE_PRESENT) == 0 {
            self.add_admin_page(table, offset);
            entry = *table.offset(offset as isize);
            if (entry & PAGE_PRESENT) == 0 {
                panic!("BUG: Admin page was not added properly");
            }
        }

        page::pte_to_vaddr(entry) as *const PageTableEntry
    }

    pub unsafe fn map(&mut self) {
        let mut current_addr = self.area_start;

        while current_addr < self.area_end {
            let mut table = self.top_level_table;
            let mut offset;

            if cfg!(target_arch = "x86_64") {
                // Extract the page directory pointer table from the page map
                // level 4 offset table
                offset = page::pml4_offset(current_addr);
                table = self.extract_entry(table, offset);
            }

            // Extract the page directory table from the page directory
            // pointer table
            offset = page::pdp_offset(current_addr);
            table = self.extract_entry(table, offset);

            // Extract the page table from the page directory
            offset = page::pd_offset(current_addr);
            table = self.extract_entry(table, offset);

            let page_table_mfn = page::vaddr_to_mfn(table as page::Vaddr);
            let new_pte = page::pfn_to_pte(self.reg_pfn_pool);
            let mut pt_offset;
            let mut page_table_addr: page::Maddr;

            pt_offset = page::pt_offset(current_addr) as usize;
            pt_offset *= size_of::<PageTableEntry>();

            page_table_addr = (page_table_mfn as page::Maddr) << PAGE_SHIFT;
            page_table_addr += pt_offset as page::Maddr;

            self.add_mmu_update(page_table_addr, new_pte as PageTableEntry,
                                false);

            self.reg_pfn_pool += 1;
            current_addr += PAGE_SIZE;
        }

        self.flush_updates();
    }
}

