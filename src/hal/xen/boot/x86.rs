use hal::xen;
use hal::arch::mmu::PageEntry;

use hal::mmu::{Vaddr, Maddr, Mfn};

use hal::xen::defs::StartInfo;
use hal::xen::memory::MapFlags;

use hal::xen::arch::x86::memory::IdentityMapper;

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

        ::hal::xen::arch::x86::traps::init();
    }
}

pub unsafe fn init_memory() -> (usize, usize) {
    raw_println!("Kernel sections (end @ -> {:p}):", &__uni_end);
    raw_println!("  .boot: {:p} - {:p}", &__boot_start, &__boot_end);
    raw_println!("  .text: {:p} - {:p}", &__text_start, &__text_end);
    raw_println!("  .rodata: {:p} - {:p}", &__rodata_start, &__rodata_end);
    raw_println!("  .data: {:p} - {:p}", &__data_start, &__data_end);

    let pt_base = Vaddr::new((*start_info).pt_base);
    let nr_pt_frames: usize = (*start_info).nr_pt_frames;
    let nr_pages: usize = (*start_info).nr_pages;

    let mut mapper = IdentityMapper::new(pt_base, nr_pt_frames, nr_pages);

    raw_println!("start info: 0x{:x}", start_info as usize);
    raw_println!("number of pages: {}", (*start_info).nr_pages);
    raw_println!("pt_base: 0x{:x}", (*start_info).pt_base);
    raw_println!("nr_pt_frames: {}", (*start_info).nr_pt_frames);
    raw_println!("Allocating heap 0x{:x}-0x{:x} ({} kB)",
                 *mapper.area_start, *mapper.area_end,
                 (*mapper.area_end - *mapper.area_start) / 1024);

    mapper.map();

    (*mapper.area_start, *mapper.area_end - *mapper.area_start)
}

unsafe fn map_shared_info() {
    let shared_info_pte = PageEntry::from(Maddr::new((*start_info).shared_info as u64));
    let shared_info_addr = Vaddr::from_ptr(&xen::shared_info);

    // Map shared info
    assert_eq!(xen::memory::update_va_mapping(shared_info_addr,
                                              shared_info_pte,
                                              MapFlags::InvlpgLocal), 0)
}
