use xen::defs::{SharedInfo, Ulong};

use xen::memory::MapFlags;
use xen::memory::update_va_mapping;

use arch::x86_common::start_info;

macro_rules! pte {
    ($x:expr) => {
        (($x as ::arch::defs::TableEntry) &
         (::arch::defs::PTE_FLAGS_MASK ^ ::core::u64::MAX)) | ::arch::defs::PAGE_FLAGS;
    }
}

mod mapper;
pub mod page;

pub unsafe fn map_shared_info() {
    let shared_info_pte = pte!((*start_info).shared_info);
    let shared_info_ptr: *const SharedInfo = &super::_shared_info;

    // Map shared info
    update_va_mapping(shared_info_ptr as Ulong, shared_info_pte,
                      MapFlags::InvlpgLocal);
}

pub unsafe fn init() -> (usize, usize) {
    let pt_base: page::Vaddr = (*start_info).pt_base;
    let nr_pt_frames: usize = (*start_info).nr_pt_frames;
    let nr_pages: usize = (*start_info).nr_pages;

    let mut mapper = mapper::IdentityMapper::new(pt_base, nr_pt_frames, nr_pages);

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
