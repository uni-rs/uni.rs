use alloc::boxed::{Box, FnBox};

use super::stack::Stack;

#[cfg(target_arch = "x86")]
mod i686;

#[cfg(target_arch = "x86_64")]
mod x86_64;

mod imp {
    #[cfg(target_arch = "x86")]
    pub use super::i686::*;

    #[cfg(target_arch = "x86_64")]
    pub use super::x86_64::*;
}

/// A thread has to be wrapped by a calling function. This function
/// is responsible to setup/cleanup for the actual closure that the user wants
/// to call. This closure is passed as first parameter to the wrapping function
/// and is named 'f' here. This pointer *IS* extracted from a Box via a call
/// to Box::into_raw. The wrapper function is now responsible for its proper
/// deallocation when the thread is terminated
pub type WrapperFn = extern "C" fn(f: *mut u8) -> !;

#[derive(Debug)]
pub struct Context {
    regs: imp::Registers,
}

impl Context {
    pub unsafe fn new<F>(wrapper: WrapperFn, mut f: F, stack: &mut Stack) -> Self
        where F: FnMut() -> (), F: Send + 'static {
        let fun = move || {
            f();
        };
        let boxed_fun: Box<FnBox()> = Box::new(fun);

        let fun_ptr = Box::into_raw(Box::new(boxed_fun)) as *mut u8;

        Context {
            regs: imp::Registers::new(wrapper, fun_ptr, stack.top().unwrap()),
        }
    }

    /// Load the current context to the CPU
    pub fn load(&self) -> ! {
        unsafe {
            imp::registers_load(&self.regs);
        }
    }

    #[inline(always)]
    #[cfg_attr(feature = "clippy", allow(inline_always))]
    /// Save the current context. This function will actually return twice.
    /// The first return is just after saving the context and will return false
    /// The second time is when the saved context gets restored and will return
    /// true
    ///
    /// This function is always inlined because otherwise the stack frame gets
    /// corrupted as this will return twice. The second time the function
    /// returns the stack frame will most likely be corrupted. To avoid that
    /// inline the function to merge the stack frame of this function with
    /// the caller's.
    pub fn save(&mut self) -> bool {
        unsafe {
            imp::registers_save(&mut self.regs)
        }
    }
}
