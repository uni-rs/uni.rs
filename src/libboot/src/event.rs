#[no_mangle]
pub unsafe extern "C" fn do_hypervisor_callback() {
    panic!("Hypercall callback");
}
