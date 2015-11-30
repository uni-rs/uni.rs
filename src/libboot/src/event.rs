// TODO: Rethink this in an architecture independent fashion in due time

use xen::event::dispatcher;
use xen::arch::x86::callbacks::set_callbacks;

extern "C" {
    fn hypervisor_callback();
    fn failsafe_callback();
}

#[cfg(target_arch = "x86")]
fn init_callbacks() {
    unsafe {
        use xen::defs::FLAT_KERNEL_CS;

        set_callbacks(FLAT_KERNEL_CS, hypervisor_callback,
                      FLAT_KERNEL_CS, failsafe_callback);
    }
}

#[cfg(target_arch = "x86_64")]
fn init_callbacks() {
    unsafe {
        set_callbacks(hypervisor_callback, failsafe_callback, None);
    }
}

#[no_mangle]
pub unsafe extern "C" fn do_hypervisor_callback() {
    panic!("Hypercall callback");
}

pub fn init() {
    init_callbacks();

    dispatcher().mask_all();

    println!("Event subsystem initialized");
}
