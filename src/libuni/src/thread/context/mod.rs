use alloc::boxed::{Box, FnBox};

use super::stack::Stack;

#[cfg(target_arch = "x86")]
mod x86;

#[cfg(target_arch = "x86_64")]
mod x86_64;

mod imp {
    #[cfg(target_arch = "x86")]
    pub use super::x86::*;

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
    pub unsafe fn empty() -> Self {
        Context {
            regs: imp::Registers::empty(),
        }
    }

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

    /// Replace the current context with ourself and save the current inside
    /// out_context.
    pub unsafe fn switch_with(&self, out_context: &mut Context) {
        imp::registers_switch(&mut out_context.regs, &self.regs);
    }
}
