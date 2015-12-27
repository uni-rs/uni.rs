use hal::xen::hypercall::hypercall2;
use hal::xen::hypercall::HypercallKind;

use hal::xen::defs::{Ulong, EvtchnPort};

mod dispatcher;

pub use self::dispatcher::Dispatcher;

static mut DISPATCHER: Dispatcher = Dispatcher::new();

// XXX: Is this really "generic" (i.e., is it present on ARM)
extern "C" {
    fn hypervisor_callback();
    fn failsafe_callback();
}

#[no_mangle]
/// This function is called when an event occur
pub unsafe extern "C" fn do_hypervisor_callback() {
    dispatcher().serve_event();
}

pub fn dispatcher<'a>() -> &'a mut Dispatcher {
    unsafe {
        &mut DISPATCHER
    }
}

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
