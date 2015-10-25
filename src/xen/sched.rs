use xen::hypercall::hypercall2;
use xen::hypercall::HyperCalls;

use ::arch::defs::Ulong;

#[allow(dead_code)]
enum SchedOp {
    Yield = 0,
    Block = 1,
    Shutdown = 2,
    Poll = 3,
    RemoteShutdown = 4,
    ShutdownCode = 5,
    Watchdog = 6,
}

#[allow(dead_code)]
fn sched_op(op: SchedOp, sched: Ulong) -> i32 {
    hypercall2(HyperCalls::SchedOp, op as Ulong, sched) as i32
}

pub fn yield_cpu() -> i32 {
    sched_op(SchedOp::Yield, 0)
}
