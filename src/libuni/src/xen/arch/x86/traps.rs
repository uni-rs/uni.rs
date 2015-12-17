use xen::defs::{TrapInfo, Ulong};

use xen::hypercall::hypercall1;
use xen::hypercall::HypercallKind;

pub unsafe fn set_trap_table(table: *const TrapInfo) {
    hypercall1(HypercallKind::SetTrapTable, table as Ulong);
}
