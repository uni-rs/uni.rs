//! Set callbacks necessary to the hypervisor to perform upcalls

use hal::xen::defs::Ulong;
use hal::xen::hypercall::HypercallKind;

pub type CallbackType = unsafe extern "C" fn() -> ();

#[cfg(target_arch = "x86")]
pub unsafe fn set_callbacks(hypervisor_callback_cs: u16,
                            hypervisor_callback: CallbackType,
                            failsafe_callback_cs: u16,
                            failsafe_callback: CallbackType) {
    use hal::xen::hypercall::hypercall4;

    let hypervisor_callback = hypervisor_callback as *const u8 as Ulong;
    let failsafe_callback = failsafe_callback as *const u8 as Ulong;

    hypercall4(HypercallKind::SetCallbacks,
               hypervisor_callback_cs as Ulong, hypervisor_callback,
               failsafe_callback_cs as Ulong, failsafe_callback);
}

#[cfg(target_arch = "x86_64")]
pub unsafe fn set_callbacks(hypervisor_callback: CallbackType,
                            failsafe_callback: CallbackType,
                            syscall_callback_opt: Option<CallbackType>) {
    use hal::xen::hypercall::hypercall3;

    let hypervisor_callback = hypervisor_callback as *const u8 as Ulong;
    let failsafe_callback = failsafe_callback as *const u8 as Ulong;
    let mut syscall_callback = 0;

    if let Some(callback) = syscall_callback_opt {
        syscall_callback = callback as *const u8 as Ulong;
    }

    hypercall3(HypercallKind::SetCallbacks, hypervisor_callback,
               failsafe_callback, syscall_callback);
}
