use defs::{TrapInfo, Ulong};

use hypercall::hypercall1;
use hypercall::HypercallKind;

pub unsafe fn set_trap_table(table: *const TrapInfo) {
    hypercall1(HypercallKind::SetTrapTable, table as Ulong);
}
