use xen::hypercall::hypercall2;
use xen::hypercall::HyperCalls;

use ::arch::defs::Ulong;

pub type EvtchnPort = u32;

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

#[repr(C)]
struct EvtchnSend {
    port: EvtchnPort,
}

fn event_channel_op(op: EventOp, event: Ulong) -> Ulong {
    hypercall2(HyperCalls::EventChannelOp, op as Ulong, event)
}

pub fn send(port: EvtchnPort) -> i32 {
    let ev: EvtchnSend = EvtchnSend {
        port: port,
    };
    let ev_ptr = &ev as *const EvtchnSend;

    event_channel_op(EventOp::Send, ev_ptr as Ulong) as i32
}
