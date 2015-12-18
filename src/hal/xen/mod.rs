use io::Write;

use hal::{
    local_irq_enable,
    local_irq_disable
};

use hal::intrinsics::wmb;

use thread::Scheduler;

mod hypercall;

pub mod defs;

pub mod boot;
pub mod memory;
pub mod event;
pub mod sched;

pub mod console;

pub mod arch;

extern "C" {
    // This symbol must be present in code using libxen
    pub static mut shared_info: self::defs::SharedInfo;
}

pub fn enable_upcalls() -> u8 {
    unsafe {
        let ret = shared_info.vcpu_info[0].evtchn_upcall_mask;

        wmb();
        shared_info.vcpu_info[0].evtchn_upcall_mask = 0;
        wmb();

        ret
    }
}

pub fn disable_upcalls() -> u8 {
    unsafe {
        let ret = shared_info.vcpu_info[0].evtchn_upcall_mask;

        shared_info.vcpu_info[0].evtchn_upcall_mask = 1;
        wmb();

        ret
    }
}

pub fn set_upcalls_state(state: u8) {
    unsafe {
        wmb();
        shared_info.vcpu_info[0].evtchn_upcall_mask = state;
        wmb();
    }
}

extern {
    fn main(_: isize, _: *const *const u8) -> isize;
}

#[no_mangle]
/// Entry point of the application called by boot assembly
pub extern "C" fn uni_rust_entry() -> ! {
    boot::init();

    println!("Uni.rs is booting");

    // Memory initialization is unsafe
    unsafe {
        let (heap_start, heap_size) = boot::init_memory();

        ::allocator::init(heap_start, heap_size);
    }

    boot::event::init();

    unsafe {
        ::console::console().init_input();
    }

    local_irq_enable();

    println!("Creating main thread");

    // Spawn main thread
    Scheduler::spawn(|| {
        let app_ret = unsafe {
            main(0, ::core::ptr::null())
        };

        local_irq_disable();

        ::console::console().flush().unwrap();

        sched::poweroff(app_ret as ::hal::xen::defs::Ulong);

        panic!("Failed to poweroff the machine !");
    });

    println!("Starting scheduler");

    Scheduler::schedule();

    unreachable!();
}
