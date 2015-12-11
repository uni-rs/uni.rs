//! Scheduling primitives for Uni.rs.

use alloc::boxed::{Box, FnBox};

use intrusive::link::Link;
use intrusive::list::{Node, Owner};

use self::stack::Stack;
use self::context::Context;

mod stack;
mod context;
mod scheduler;

const DEFAULT_STACK_SIZE: usize = 8192;

pub struct Thread {
    t_impl: Box<ThreadImpl>,
}

impl Thread {
    pub fn spawn<F>(fun: F) -> Thread where F: Fn() -> () {
        Builder::new().spawn(fun)
    }
}

pub struct Builder {
    stack_size: usize,
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn stack_size(mut self, stack_size: usize) -> Builder {
        self.stack_size = stack_size;
        self
    }

    pub fn spawn<F>(self, fun: F) -> Thread where F: Fn() -> () {
        let thread_impl = Box::new(ThreadImpl::new(fun, self.stack_size));

        Thread {
            t_impl: thread_impl,
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            stack_size: DEFAULT_STACK_SIZE,
        }
    }
}

#[derive(PartialEq)]
enum State {
    Empty,
    Ready,
    Running,
    Blocked,
    Terminated,
}

struct ThreadImpl {
    context: Context,
    state: State,
    // On Drop stack is released
    #[allow(dead_code)]
    stack: Stack,
    prev: Link<ThreadImpl>,
    next: Link<ThreadImpl>,
}

impl ThreadImpl {
    pub unsafe fn empty() -> Self {
        ThreadImpl {
            context: Context::empty(),
            state: State::Empty,
            stack: Stack::null(),
            prev: Link::none(),
            next: Link::none(),
        }
    }

    pub fn new<F>(fun: F, stack_size: usize) -> Self where F: Fn() -> () {
        let mut stack = unsafe { Stack::new(stack_size) };

        if stack.is_null() {
            panic!("Cannot allocate a stack (size: {}) for a new thread",
                   stack_size);
        }

        ThreadImpl {
            context: unsafe { Context::new(thread_wrapper, fun, &mut stack) },
            state: State::Ready,
            stack: stack,
            prev: Link::none(),
            next: Link::none(),
        }
    }

    pub unsafe fn yield_to(&mut self, thread: &ThreadImpl) {
        thread.context.switch_with(&mut self.context);
    }
}

extern "C" fn thread_wrapper(f: *mut u8) -> ! {
    {
        let boxed_fn: Box<Box<FnBox()>> = unsafe {
            Box::from_raw(f as *mut Box<FnBox()>)
        };

        boxed_fn();
    }

    unimplemented!();
}

impl Node for ThreadImpl {
    #[inline]
    fn prev(&self) -> &Link<Self> {
        &self.prev
    }

    #[inline]
    fn next(&self) -> &Link<Self> {
        &self.next
    }

    #[inline]
    fn prev_mut(&mut self) -> &mut Link<Self> {
        &mut self.prev
    }

    #[inline]
    fn next_mut(&mut self) -> &mut Link<Self> {
        &mut self.prev
    }
}

// This trait cannot be implemented for Box in libintrusive as it must not
// depend on rust's liballoc
type ThreadImplBox = Box<ThreadImpl>;

impl Owner<ThreadImpl> for ThreadImplBox {
    #[inline]
    fn get(&self) -> &ThreadImpl {
        &**self
    }

    #[inline]
    fn get_mut(&mut self) -> &mut ThreadImpl {
        &mut **self
    }

    #[inline]
    fn take(self) {
        Box::into_raw(self);
    }

    unsafe fn from_raw_ptr(ptr: *mut ThreadImpl) -> Self {
        Box::from_raw(ptr)
    }
}
