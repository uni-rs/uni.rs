use core::mem;

use hal::mmu::Vaddr;
use hal::arch::PageEntry;

use hal::xen::hypercall::{hypercall3, hypercall4};
use hal::xen::hypercall::HypercallKind;

use hal::xen::defs::{Ulong, DOMID_SELF};

pub enum MapFlags {
    None = 0,
    FlushLocal = 1,
    InvlpgLocal = 2,
    FlushAll = 5,
    InvlpgAll = 4,
}

pub unsafe fn update_va_mapping(guest_page: Vaddr, entry: PageEntry,
                                flags: MapFlags) -> i32 {
    if mem::size_of::<Ulong>() == mem::size_of::<u64>() {
        hypercall3(HypercallKind::UpdateVaMapping, *guest_page,
                   entry.value() as Ulong, flags as Ulong) as i32
    } else {
        hypercall4(HypercallKind::UpdateVaMapping, *guest_page,
                   entry.value() as Ulong, (entry.value() >> 32) as Ulong,
                   flags as Ulong) as i32
    }
}

#[repr(C)]
pub struct MmuUpdate {
    ptr: u64,
    val: u64,
}

impl Copy for MmuUpdate {}
impl Clone for MmuUpdate {
    fn clone(&self) -> MmuUpdate {
        *self
    }
}

impl MmuUpdate {
    pub const fn null() -> MmuUpdate {
        MmuUpdate {
            ptr: 0,
            val: 0,
        }
    }

    pub const fn new(ptr: u64, val: u64) -> MmuUpdate {
        MmuUpdate {
            ptr: ptr,
            val: val,
        }
    }
}

pub unsafe fn mmu_update(updates: *const MmuUpdate, count: usize,
                         done_out: *mut u32) -> i32 {
    hypercall4(HypercallKind::MmuUpdate, updates as Ulong, count as Ulong,
               done_out as Ulong, DOMID_SELF as Ulong) as i32
}
