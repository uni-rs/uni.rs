use core::mem::size_of;

use xen::SharedInfo;

use arch::defs::Ulong;

use xen::hypercall::HyperCalls;
use xen::hypercall::hypercall3;
use xen::hypercall::hypercall4;

use arch::x86_common::start_info;

macro_rules! pte {
    ($x:expr) => {
        (($x as ::arch::defs::TableEntry) &
         (::arch::defs::PTE_FLAGS_MASK ^ ::core::u64::MAX)) | ::arch::defs::PAGE_FLAGS;
    }
}

pub mod page;

#[allow(dead_code)]
enum MapFlags {
    None = 0,
    FlushLocal = 1,
    InvlpgLocal = 2,
    FlushAll = 5,
    InvlpgAll = 4,
}

fn update_va_mapping(guest_page: Ulong, mac_page: page::Maddr,
                     flags: MapFlags) -> i32 {
    if size_of::<Ulong>() == size_of::<u64>() {
        hypercall3(HyperCalls::UpdateVaMapping, guest_page, mac_page as Ulong,
                   flags as Ulong) as i32
    } else {
        hypercall4(HyperCalls::UpdateVaMapping, guest_page,
                   mac_page as Ulong, (mac_page >> 32) as Ulong,
                   flags as Ulong) as i32
    }
}

pub unsafe fn map_shared_info() {
    let shared_info_pte = pte!((*start_info).shared_info);
    let shared_info_ptr: *const SharedInfo = &::xen::_shared_info;

    // Map shared info
    update_va_mapping(shared_info_ptr as Ulong, shared_info_pte,
                      MapFlags::InvlpgLocal);
}
