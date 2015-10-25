use xen::hypercall::hypercall2;
use xen::hypercall::HyperCalls;

use ::arch::defs::Ulong;

#[allow(dead_code)]
enum EventOp {
    BindInterdomain = 0,
    BindVirq = 1,
    BindPirq = 2,
    Close = 3,
    Send = 4,
    Status = 5,
    AllocUnbound = 6,
    BindIpi = 7,
    BindVcpu = 8,
    Unmask = 9,
    Reset = 10,
    InitControl = 11,
    ExpandArray = 12,
    SetPriority = 13,
}

#[allow(dead_code)]
fn event_channel_op(op: EventOp, event: Ulong) -> Ulong {
    hypercall2(HyperCalls::EventChannelOp, op as Ulong, event)
}
