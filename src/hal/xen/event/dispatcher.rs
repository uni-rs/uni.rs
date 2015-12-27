//! Main utility to dispatch and manage Xen events

use core::ptr::null_mut;

use core::intrinsics::atomic_xchg;

use hal::xen::shared_info;

use hal::xen::defs::EvtchnPort;
use hal::xen::defs::ULONG_SIZE;

use hal::arch::utils::{first_bit, wmb};
use hal::arch::utils::{atomic_set_bit, atomic_clear_bit};

const NUMBER_OF_EVENTS: usize = 1024;

pub type EventHandler = fn(port: EvtchnPort, data: *mut u8) -> ();

#[derive(Clone, Copy)]
struct EventData {
    pub handler: EventHandler,
    pub data: *mut u8,
}

impl EventData {
    pub const fn new(handler: EventHandler, data: *mut u8) -> Self {
        EventData {
            handler: handler,
            data: data,
        }
    }
}

pub struct Dispatcher {
    handlers: [EventData; NUMBER_OF_EVENTS],
}

impl Dispatcher {
    pub fn default_handler(port: EvtchnPort, _data: *mut u8) {
        panic!("Unhandled port ({})", port);
    }

    pub const fn new() -> Self {
        Dispatcher {
            handlers: [EventData::new(Dispatcher::default_handler, null_mut());
                       NUMBER_OF_EVENTS],
        }
    }

    #[inline(always)]
    pub unsafe fn serve_event(&self) {
        let cpu = &mut shared_info.vcpu_info[0];

        cpu.evtchn_upcall_pending = 0;

        wmb();

        let mut pending_sel = atomic_xchg(&mut cpu.evtchn_pending_sel, 0);

        while pending_sel != 0 {
            let next_event_offset = first_bit(pending_sel);

            pending_sel &= !(1 << next_event_offset);

            let mut event = shared_info.evtchn_pending[next_event_offset] &
                            !shared_info.evtchn_mask[next_event_offset];

            while event != 0 {
                let event_offset = first_bit(event);
                let port = (next_event_offset * ULONG_SIZE * 8) + event_offset;

                event &= !(1 << event_offset);

                if port >= NUMBER_OF_EVENTS {
                    panic!("Event occurred for out of bound port ({})", port);
                }

                (self.handlers[port].handler)(port as EvtchnPort,
                                              self.handlers[port].data);

                atomic_clear_bit(port, &mut shared_info.evtchn_pending[0]);
            }
        }
    }

    pub fn bind_port(&mut self, port: EvtchnPort, handler: EventHandler,
                     data: *mut u8) {
        if self.handlers[port as usize].handler !=
            Dispatcher::default_handler {
            panic!("Trying to override non default handler for port {}", port);
        }

        self.handlers[port as usize] = EventData::new(handler, data);
    }

    pub fn mask_all(&self) {
        let mut i: EvtchnPort = 0;

        while (i as usize) < NUMBER_OF_EVENTS {
            self.mask_event(i);

            i += 1;
        }
    }

    pub fn mask_event(&self, port: EvtchnPort) {
        unsafe {
            if port < (NUMBER_OF_EVENTS as EvtchnPort) {
                atomic_set_bit(port as usize, &mut shared_info.evtchn_mask[0]);
            }
        }
    }

    pub fn unmask_event(&self, port: EvtchnPort) {
        unsafe {
            if port < (NUMBER_OF_EVENTS as EvtchnPort) {
                atomic_clear_bit(port as usize,
                                 &mut shared_info.evtchn_mask[0]);
            }
        }
    }
}
