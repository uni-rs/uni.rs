use core::ptr;
use core::mem::size_of;

use rlibc::memset;

use hal::mmu;

use hal::arch::defs::PAGE_SIZE;

use hal::arch::mmu::{PageEntry, PageFlags};
use hal::arch::mmu::{PTE_PER_TABLE};

use hal::xen::memory::{MmuUpdate, MapFlags};
use hal::xen::memory::{mmu_update, update_va_mapping};

macro_rules! div_up {
    ($x:expr, $y:expr) => {
        ($x - 1) / $y + 1 + (if $x % $y == 0 { 1 } else { 0 })
    }
}

const MAX_UPDATES: usize = 100;

/// Helper that creates an identity mapping between physical memory and virtual
/// memory.
pub struct IdentityMapper {
    top_level_table: *const PageEntry,
    admin_pfn_pool: mmu::Pfn,
    reg_pfn_pool: mmu::Pfn,
    updates: [MmuUpdate; 100],
    nr_update: usize,
    pub area_start: mmu::Vaddr,
    pub area_end: mmu::Vaddr,
}

impl IdentityMapper {
    pub fn new(pt_base: mmu::Vaddr, nr_pt_frames: usize,
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

        let pfn_base = mmu::Pfn::from(pt_base) + 1;
        let reg_pfn_base = pfn_base + nr_pt_frames + admin_page_count;
        let nr_page_to_map = nr_pages - *reg_pfn_base;

        let area_start = mmu::Vaddr::from(reg_pfn_base);

        IdentityMapper {
            top_level_table: unsafe { pt_base.as_ptr() },
            admin_pfn_pool: pfn_base + nr_pt_frames,
            reg_pfn_pool: reg_pfn_base,
            updates: [MmuUpdate::null(); 100],
            nr_update: 0,
            area_start: area_start,
            area_end: unsafe { area_start.incr(nr_page_to_map * PAGE_SIZE) },
        }
    }

    // Xen will prevent a page from being a page table (or higher level
    // table) if this page is mapped somewhere as writable or if it has
    // some non valid entries. This is why we first zero the page, then
    // map it as read only
    unsafe fn add_admin_page(&mut self, table: *const PageEntry,
                             offset: usize) {
        let new_entry_pfn = self.admin_pfn_pool;
        let new_entry_mfn = mmu::Mfn::from(new_entry_pfn);
        let mut new_entry_vaddr = mmu::Vaddr::from(new_entry_pfn);
        let new_entry_pte = PageEntry::from(new_entry_pfn);
        let new_entry_maddr = mmu::Maddr::from(new_entry_mfn);

        // We map this new page in our virtual address space so that we
        // can clear it
        update_va_mapping(new_entry_vaddr, new_entry_pte,
                          MapFlags::InvlpgLocal);

        // Clear the page before mapping it
        memset(new_entry_vaddr.as_mut_ptr(), 0, PAGE_SIZE);

        // We remap it as read only
        update_va_mapping(new_entry_vaddr,
                          PageEntry::new(*new_entry_maddr).set(PageFlags::Present),
                          MapFlags::InvlpgLocal);

        let table_mfn = mmu::Mfn::from(mmu::Vaddr::from_ptr(table));
        let table_mach = mmu::Maddr::from(table_mfn).incr(offset *
                                                          size_of::<PageEntry>());

        // We add a new page to the table
        self.add_mmu_update(table_mach, new_entry_pte, true);

        self.admin_pfn_pool += 1;
    }

    unsafe fn add_mmu_update(&mut self, ptr: mmu::Maddr, val: PageEntry,
                             force_update: bool) {
        self.updates[self.nr_update as usize] = MmuUpdate::new(*ptr, val.value());

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

    unsafe fn extract_entry(&mut self, table: *const PageEntry,
                            offset: usize) -> *const PageEntry {
        let mut entry = *table.offset(offset as isize);

        if !entry.has(PageFlags::Present) {
            self.add_admin_page(table, offset);
            entry = *table.offset(offset as isize);
            if !entry.has(PageFlags::Present) {
                panic!("BUG: Admin page was not added properly");
            }
        }

        mmu::Vaddr::from(entry).as_ptr()
    }

    pub unsafe fn map(&mut self) {
        let mut current_addr = self.area_start;

        while current_addr < self.area_end {
            let mut table = self.top_level_table;
            let mut offset;

            if cfg!(target_arch = "x86_64") {
                // Extract the page directory pointer table from the page map
                // level 4 offset table
                offset = current_addr.l4_offset();
                table = self.extract_entry(table, offset);
            }

            // Extract the page directory table from the page directory
            // pointer table
            offset = current_addr.l3_offset();
            table = self.extract_entry(table, offset);

            // Extract the page table from the page directory
            offset = current_addr.l2_offset();
            table = self.extract_entry(table, offset);

            let page_table_mfn = mmu::Mfn::from(mmu::Vaddr::from_ptr(table));
            let new_pte = PageEntry::from(self.reg_pfn_pool);
            let pt_value = current_addr.l1_offset() * size_of::<PageEntry>();
            let page_table_addr;

            page_table_addr = mmu::Maddr::from(page_table_mfn).incr(pt_value);

            self.add_mmu_update(page_table_addr, new_pte as PageEntry,
                                false);

            self.reg_pfn_pool += 1;

            current_addr = current_addr.incr(PAGE_SIZE);
        }

        self.flush_updates();
    }
}

