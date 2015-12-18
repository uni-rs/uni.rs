use hal::xen;

use hal::xen::memory::MapFlags;

use hal::xen::defs::{Ulong, StartInfo, SharedInfo, ConsoleInterface};

#[macro_use]
mod page;
mod mapper;

pub mod traps;
pub mod defs;

extern {
    // Start info is not present on all architecture, this is why this
    // was made a global variable only for x86_*
    pub static start_info: *const StartInfo;
}

pub fn init() {
    unsafe {
        let console_vaddr: self::page::Vaddr;

        map_shared_info();

        console_vaddr = page::mfn_to_vaddr((*start_info).domu_console.mfn);

        ::console::init(console_vaddr as *mut ConsoleInterface,
                           (*start_info).domu_console.evtchn);

        self::traps::init();
    }
}

pub unsafe fn init_memory() -> (usize, usize) {
    let pt_base: page::Vaddr = (*start_info).pt_base;
    let nr_pt_frames: usize = (*start_info).nr_pt_frames;
    let nr_pages: usize = (*start_info).nr_pages;

    let mut mapper = mapper::IdentityMapper::new(pt_base, nr_pt_frames,
                                                 nr_pages);

    println!("start info: 0x{:x}", start_info as usize);
    println!("number of pages: {}", (*start_info).nr_pages);
    println!("pt_base: 0x{:x}", (*start_info).pt_base);
    println!("nr_pt_frames: {}", (*start_info).nr_pt_frames);

    println!("Allocating heap 0x{:x}-0x{:x} ({} kB)", mapper.area_start,
             mapper.area_end,
             (mapper.area_end - mapper.area_start) / 1024);

    mapper.map();

    (mapper.area_start, mapper.area_end - mapper.area_start)
}

unsafe fn map_shared_info() {
    let shared_info_pte = pte!((*start_info).shared_info);
    let shared_info_ptr: *const SharedInfo = &xen::shared_info;

    // Map shared info
    assert_eq!(xen::memory::update_va_mapping(shared_info_ptr as Ulong,
                                              shared_info_pte.value(),
                                              MapFlags::InvlpgLocal), 0)
}
