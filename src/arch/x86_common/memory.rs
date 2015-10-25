use core::mem::size_of;

use ::xen::SharedInfo;

use ::arch::defs::Ulong;
use ::arch::defs::MAX_ULONG;

use ::xen::hypercall::HyperCalls;
use ::xen::hypercall::hypercall3;
use ::xen::hypercall::hypercall4;

use ::arch::x86_common::start_info;

const PAGE_SHIFT: u32 = 12;
const PAGE_SIZE: u32 = 1 << PAGE_SHIFT;

const PTE_MASK: Ulong = (PAGE_SIZE as Ulong) - 1;

pub type Vaddr = Ulong;
pub type Pfn = Ulong;
pub type Mfn = Ulong;

macro_rules! pte {
    ($x:expr) => {
        (($x as Ulong) & (PTE_MASK ^ MAX_ULONG)) | 3;
    }
}

#[allow(dead_code)]
enum MapFlags {
    None = 0,
    FlushLocal = 1,
    InvlpgLocal = 2,
    FlushAll = 5,
    InvlpgAll = 4,
}

fn update_va_mapping(guest_page: Ulong, mac_page: Ulong,
                     flags: MapFlags) -> i32 {
    if size_of::<Ulong>() == size_of::<u64>() {
        hypercall3(HyperCalls::UpdateVaMapping, guest_page, mac_page,
                   flags as Ulong) as i32
    } else {
        hypercall4(HyperCalls::UpdateVaMapping, guest_page,
                   mac_page, 0, flags as Ulong) as i32
    }
}

pub unsafe fn map_shared_info() {
    let shared_info_pte = pte!((*start_info).shared_info);
    let shared_info_ptr: *const SharedInfo = &::xen::_shared_info;

    // Map shared info
    update_va_mapping(shared_info_ptr as Ulong, shared_info_pte,
                      MapFlags::InvlpgLocal);
}
