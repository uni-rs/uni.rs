use alloc_uni;

use core::ptr;
use core::mem::size_of;

use rlibc::memset;

use hal::arch::defs::PAGE_SIZE;

use hal::mmu::{Vaddr, Maddr, Pfn, Mfn};

use hal::arch::mmu::{PageEntry, PageFlags};

use hal::xen::boot::start_info;

use hal::xen::memory::{MmuUpdate, MapFlags};
use hal::xen::memory::{mmu_update, update_va_mapping};

/// Add an admin frame (ex: add a new page table)
unsafe fn add_admin_frame(table: *const PageEntry,
                          offset: usize) -> Result<(), i32> {
    let admin_page = alloc_uni::__rust_allocate(PAGE_SIZE, PAGE_SIZE);

    if admin_page == ptr::null_mut() {
        panic!("Cannot allocate a new admin page");
    }

    // Clear the page before mapping it
    memset(admin_page, 0, PAGE_SIZE);

    let admin_page_vaddr = Vaddr::from_ptr(admin_page);
    let admin_page_pfn = Pfn::from(admin_page_vaddr);
    let admin_page_pte = PageEntry::from(admin_page_pfn);
    let admin_page_mfn = Mfn::from(admin_page_pfn);
    let admin_page_maddr = Maddr::from(admin_page_mfn);

    let ret = update_va_mapping(admin_page_vaddr,
                                PageEntry::new(*admin_page_maddr).set(PageFlags::Present),
                                MapFlags::InvlpgLocal);
    if ret < 0 {
        return Err(ret);
    }

    let table_mfn = Mfn::from(Vaddr::from_ptr(table));
    let table_mach = Maddr::from(table_mfn).incr(offset * size_of::<PageEntry>());

    let update = MmuUpdate::new(*table_mach, admin_page_pte.value());

    let ret = mmu_update(&update, 1, ptr::null_mut());

    if ret < 0 {
        Err(ret)
    } else {
        Ok(())
    }
}

unsafe fn extract_table_entry(table: *const PageEntry,
                              offset: usize) -> Result<*const PageEntry, i32> {
    let mut entry = *table.offset(offset as isize);

    if !entry.has(PageFlags::Present) {
        try!(add_admin_frame(table, offset));

        entry = *table.offset(offset as isize);
        if !entry.has(PageFlags::Present) {
            panic!("BUG: Admin frame was not added properly");
        }
    }

    Ok(Vaddr::from(entry).as_ptr())
}

/// Map a single physical page into virtual address space
pub unsafe fn map_page(addr: Vaddr, mfn: Mfn) -> Result<(), i32> {
    let mut table = (*start_info).pt_base as *const PageEntry;
    let mut offset;

    if cfg!(target_arch = "x86_64") {
        // Extract the page directory pointer table from the page map
        // level 4 offset table
        offset = addr.l4_offset();
        table = try!(extract_table_entry(table, offset));
    }

    // Extract the page directory table from the page directory
    // pointer table
    offset = addr.l3_offset();
    table = try!(extract_table_entry(table, offset));

    // Extract the page table from the page directory
    offset = addr.l2_offset();
    table = try!(extract_table_entry(table, offset));

    let page_table_mfn = Mfn::from(Vaddr::from_ptr(table));
    let new_pte = PageEntry::from(mfn);
    let pt_value = addr.l1_offset() * size_of::<PageEntry>();
    let page_table_addr = Maddr::from(page_table_mfn).incr(pt_value);

    let update = MmuUpdate::new(*page_table_addr, new_pte.value());

    let ret = mmu_update(&update, 1, ptr::null_mut());

    if ret < 0 {
        Err(ret)
    } else {
        Ok(())
    }
}

/// Map `count` physical pages into virtual address space
pub unsafe fn map_contiguous(mut addr: Vaddr, mut pfn_base: Pfn,
                             count: usize) -> Result<(), i32> {
    for _ in 0..count {
        try!(map_page(addr, Mfn::from(pfn_base)));

        pfn_base += 1;
        addr = addr.incr(PAGE_SIZE);
    }

    Ok(())
}

/// Map non contiguous machine frames into virtual address space
pub unsafe fn map_non_contiguous_mfn(mut addr: Vaddr,
                                     mfn_list: &[Mfn]) -> Result<(), i32> {
    for &mfn in mfn_list {
        try!(map_page(addr, mfn));

        addr = addr.incr(PAGE_SIZE);
    }

    Ok(())
}
