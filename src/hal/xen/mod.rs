//! Xen backend for Uni.rs
//!
//! Note: This backend is enabled by the feature named *xen* and thus might
//! not be available depending on your build's configuration.

use io::Write;

use hal;

use hal::arch::utils::wmb;

use thread::Scheduler;

// In order to use stdout, heap must be initialized, so "raw" console is used
// before that
macro_rules! raw_println {
    ($fmt:expr) => {
        raw_print!(concat!($fmt, "\r\n"))
    };
    ($fmt:expr, $($arg:tt)*) => {
        raw_print!(concat!($fmt, "\r\n"), $($arg)*)
    }
}

macro_rules! raw_print {
    ($($arg:tt)*) => {{
        use $crate::io::Write;

        $crate::hal::console().write_fmt(format_args!($($arg)*)).unwrap();
    }}
}

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

    raw_println!("Uni.rs is booting");

    // Memory initialization is unsafe
    unsafe {
        let (heap_start, heap_size) = boot::init_memory();

        ::allocator::init(heap_start, heap_size);
    }

    event::init();

    unsafe {
        console::console().init_input();
    }

    hal::local_irq_enable();

    // Spawn main thread
    Scheduler::spawn(|| {
        println!("Main thread started");
        println!("Uni.rs is now ready");
        println!("Control is now transfered to the application");
        println!("");

        let app_ret = unsafe {
            main(0, ::core::ptr::null())
        };

        hal::local_irq_disable();

        hal::console().flush().unwrap();

        hal::app::exit(app_ret);

        panic!("Failed to poweroff the machine !");
    });

    Scheduler::schedule();

    unreachable!();
}
