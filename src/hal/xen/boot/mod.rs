pub mod event;
pub mod arch;

use hal::{local_irq_enable, local_irq_disable};

extern {
    fn main(_: isize, _: *const *const u8) -> isize;
}

use thread::Scheduler;

// 8KB
const STACK_SIZE: usize = 8192;

#[no_mangle]
#[allow(non_upper_case_globals)]
#[link_section=".stack"]
pub static rust_stack: [u8; STACK_SIZE] = [0; STACK_SIZE];

#[no_mangle]
pub extern "C" fn uni_rust_entry() -> ! {
    self::arch::init();

    println!("Uni.rs is booting");

    // Memory initialization is unsafe
    unsafe {
        let (heap_start, heap_size) = arch::init_memory();

        ::allocator::init(heap_start, heap_size);
    }

    event::init();

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

        ::console::console().flush();

        ::hal::xen::sched::poweroff(app_ret as ::hal::xen::defs::Ulong);

        panic!("Failed to poweroff the machine !");
    });

    println!("Starting scheduler");

    Scheduler::schedule();

    unreachable!();
}
