use hal::xen::defs::{TrapInfo, Ulong};

use hal::xen::hypercall::hypercall1;
use hal::xen::hypercall::HypercallKind;

pub unsafe fn set_trap_table(table: *const TrapInfo) {
    hypercall1(HypercallKind::SetTrapTable, table as Ulong);
}
