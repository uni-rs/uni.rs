use hal::xen;
use hal::arch::PageEntry;

use hal::mmu::{Vaddr, Maddr, Mfn};

use hal::xen::defs::StartInfo;
use hal::xen::memory::MapFlags;

mod mapper;

pub mod traps;

extern {
    // Start info is not present on all architecture, this is why this
    // was made a global variable only for x86_*
    pub static start_info: *const StartInfo;
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

        self::traps::init();
    }
}

pub unsafe fn init_memory() -> (usize, usize) {
    let pt_base = Vaddr::new((*start_info).pt_base);
    let nr_pt_frames: usize = (*start_info).nr_pt_frames;
    let nr_pages: usize = (*start_info).nr_pages;

    let mut mapper = mapper::IdentityMapper::new(pt_base, nr_pt_frames,
                                                 nr_pages);

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
