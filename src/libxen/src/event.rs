use hypercall::hypercall2;
use hypercall::HypercallKind;

use defs::{Ulong, EvtchnPort};

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

#[inline]
fn event_channel_op(op: EventOp, event: Ulong) -> i32 {
    unsafe {
        hypercall2(HypercallKind::EventChannelOp, op as Ulong, event) as i32
    }
}

pub fn send(port: EvtchnPort) -> i32 {
    let ev: EvtchnSend = EvtchnSend {
        port: port,
    };
    let ev_ptr = &ev as *const EvtchnSend;

    event_channel_op(EventOp::Send, ev_ptr as Ulong)
}
