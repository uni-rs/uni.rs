use hypercall::hypercall2;
use hypercall::HypercallKind;

use defs::Ulong;

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
enum ShutdownReason {
    PowerOff = 0,
    Reboot = 1,
    Suspend = 2,
    Crash = 3,
    Watchdog = 4,
}

#[repr(C)]
struct SchedShutdown {
    reason: u32,
}

#[inline]
fn sched_op(op: SchedOp, sched: Ulong) -> i32 {
    unsafe {
        hypercall2(HypercallKind::SchedOp, op as Ulong, sched) as i32
    }
}

pub fn yield_cpu() -> i32 {
    sched_op(SchedOp::Yield, 0)
}

pub fn block() -> i32 {
    sched_op(SchedOp::Block, 0)
}

fn shutdown(reason: ShutdownReason) -> i32 {
    let r = SchedShutdown {
        reason: reason as u32,
    };

    sched_op(SchedOp::Shutdown, &r as *const SchedShutdown as Ulong)
}

fn shutdown_code(status: Ulong) -> i32 {
    sched_op(SchedOp::ShutdownCode, status)
}

pub fn crash() -> i32 {
    shutdown(ShutdownReason::Crash)
}

pub fn poweroff(status: Ulong) -> i32 {
    let ret: i32 = shutdown_code(status);

    if ret < 0 {
        return ret
    }

    shutdown(ShutdownReason::PowerOff)
}

