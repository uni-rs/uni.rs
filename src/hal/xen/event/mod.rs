//! Implementation of Xen's event layer

use hal::xen::hypercall::hypercall2;
use hal::xen::hypercall::HypercallKind;

use hal::xen::defs::{Ulong, EvtchnPort};

mod dispatcher;

pub use self::dispatcher::{dispatcher, do_hypervisor_callback};

pub use self::dispatcher::Dispatcher;

// XXX: Is this really "generic" (i.e., is it present on ARM)
extern "C" {
    fn hypervisor_callback();
    fn failsafe_callback();
}

#[doc(hidden)]
/// Init the event subsystem
pub fn init() {
    init_callbacks();

    dispatcher().mask_all();

    println!("Event subsystem initialized");
}

#[cfg(target_arch = "x86")]
fn init_callbacks() {
    unsafe {
        use hal::xen::defs::FLAT_KERNEL_CS;
        use hal::xen::arch::x86::callbacks::set_callbacks;

        set_callbacks(FLAT_KERNEL_CS, hypervisor_callback,
                      FLAT_KERNEL_CS, failsafe_callback);
    }
}

#[cfg(target_arch = "x86_64")]
fn init_callbacks() {
    unsafe {
        use hal::xen::arch::x86::callbacks::set_callbacks;

        set_callbacks(hypervisor_callback, failsafe_callback, None);
    }
}

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
fn event_channel_op<T>(op: EventOp, event: *mut T) -> i32 {
    unsafe {
        hypercall2(HypercallKind::EventChannelOp, op as Ulong,
                   event as Ulong) as i32
    }
}

/// Send an event to the remote end of the channel whose local endpoint is
/// `port`
pub fn send(port: EvtchnPort) -> i32 {
    let mut ev: EvtchnSend = EvtchnSend {
        port: port,
    };

    event_channel_op(EventOp::Send, &mut ev)
}
