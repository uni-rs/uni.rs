use alloc_uni;

use hal::xen;
use hal::arch::defs::PAGE_SIZE;
use hal::arch::mmu::PageEntry;

use hal::mmu::{Vaddr, Maddr, Pfn, Mfn};

use hal::xen::defs::StartInfo;
use hal::xen::store::XenStore;
use hal::xen::memory::MapFlags;

use hal::xen::arch::x86::memory::map_contiguous;

macro_rules! align {
    ( $val:expr, $align:expr ) => {
        $val & !($align - 1)
    }
}

// 1 Mb
const MB: usize = 0x100000;
const PFN_PER_MB: usize = MB / PAGE_SIZE;

extern {
    // Start info is not present on all architecture, this is why this
    // was made a global variable only for x86_*
    pub static start_info: *const StartInfo;

    // Limits of the boot section
    pub static __boot_start: u8;
    pub static __boot_end: u8;

    // Limits of the test section
    pub static __text_start: u8;
    pub static __text_end: u8;

    // Limits of the test section
    pub static __rodata_start: u8;
    pub static __rodata_end: u8;

    // Limits of the test section
    pub static __data_start: u8;
    pub static __data_end: u8;

    // End address of Uni.rs code and data aligned on 4Kb
    pub static __uni_end : u8;
}

pub fn init() {
    unsafe {
        let mut console_vaddr;
        let mut store_vaddr;

        map_shared_info();

        console_vaddr = Vaddr::from(Mfn::new((*start_info).domu_console.mfn));
        store_vaddr = Vaddr::from(Mfn::new((*start_info).store_mfn));

        ::hal::xen::console::init(console_vaddr.as_mut_ptr(),
                                  (*start_info).domu_console.evtchn);
        XenStore::init(store_vaddr.as_mut_ptr(), (*start_info).store_evtchn);

        ::hal::xen::arch::x86::traps::init();
    }
}

#[cfg_attr(feature = "clippy", allow(identity_op))]
pub fn init_memory() -> Vaddr {
    raw_println!("Kernel sections (end @ -> {:p}):", &__uni_end);
    raw_println!("  .boot: {:p} - {:p}", &__boot_start, &__boot_end);
    raw_println!("  .text: {:p} - {:p}", &__text_start, &__text_end);
    raw_println!("  .rodata: {:p} - {:p}", &__rodata_start, &__rodata_end);
    raw_println!("  .data: {:p} - {:p}", &__data_start, &__data_end);

    if &__uni_end as *const u8 > 0x100000 as *const u8 {
        panic!("TODO: Kernel bigger that 1Mb");
    }

    // By default Xen maps the first 4Mb so we can safely add the regions to
    // the allocator. This initialization is necessary as the memory map
    // function will use pages directly from the heap as admin pages.
    unsafe {
        alloc_uni::add_block((1 * MB) as *mut u8);
        alloc_uni::add_block((2 * MB) as *mut u8);
        alloc_uni::add_block((3 * MB) as *mut u8);
    }

    let mut cur = Vaddr::new(0x400000);
    let mut pfn_cur = Pfn::from(cur);
    let vaddr_base;
    let pfn_limit;
    let vaddr_limit;

    unsafe {
        vaddr_base = Vaddr::new((*start_info).pt_base);
        pfn_limit = Pfn::from(vaddr_base) + (*start_info).nr_pt_frames +
                    (*start_info).nr_pages;
        vaddr_limit = Vaddr::new(align!(*Vaddr::from(pfn_limit), MB));
    }

    raw_println!("Heap is 0x100000 - 0x{:x}", *vaddr_limit);

    // We map all the memory after 4Mb by 1Mb block and we inject these blocks
    // into the allocator
    while cur < vaddr_limit {
        unsafe {
            map_contiguous(cur, pfn_cur, PFN_PER_MB)
                .expect("Fail to initial memory mapping");

            alloc_uni::add_block(*cur as *mut u8);

            pfn_cur += PFN_PER_MB;
            cur = cur.incr(1 * MB);
        }
    }

    vaddr_limit
}

unsafe fn map_shared_info() {
    let shared_info_pte = PageEntry::from(Maddr::new((*start_info).shared_info as u64));
    let shared_info_addr = Vaddr::from_ptr(&xen::shared_info);

    // Map shared info
    assert_eq!(xen::memory::update_va_mapping(shared_info_addr,
                                              shared_info_pte,
                                              MapFlags::InvlpgLocal), 0)
}
