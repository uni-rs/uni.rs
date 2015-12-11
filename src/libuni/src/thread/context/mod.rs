#[cfg(target_arch = "x86")]
mod x86;

#[cfg(target_arch = "x86_64")]
mod x86_64;

/// A thread has to be wrapped by a calling function. This function
/// is responsible to setup/cleanup for the actual closure that the user wants
/// to call. This closure is passed as first parameter to the wrapping function
/// and is named 'f' here. This pointer *IS* extracted from a Box via a call
/// to Box::into_raw. The wrapper function is now responsible for its proper
/// deallocation when the thread is terminated
pub type WrapperFn = extern "C" fn(f: *mut u8) -> !;
