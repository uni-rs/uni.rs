//! Scheduler used in Uni.rs

use core::ptr;
use core::mem;

use alloc::boxed::Box;

use sync::spin::InterruptSpinLock;
use intrusive::queue::Queue;

use super::{Thread, Builder, ThreadImpl, State};

static mut SCHEDULER: SchedulerImpl = SchedulerImpl::new();

pub struct Scheduler;

impl Scheduler {
    /// Mark a thread as ready, and queue it to the ready queue of the
    /// scheduler
    pub fn ready(thread: Thread) {
        unsafe {
            SCHEDULER.ready(thread);
        }
    }

    /// Run the scheduler. The current running process is put back in the
    /// ready queue. The first element of the queue is then executed
    pub fn schedule() {
        unsafe {
            SCHEDULER.schedule();
        }
    }

    /// Terminate the execution of the current thread
    pub fn terminate() -> ! {
        unsafe {
            SCHEDULER.terminate();
        }
    }

    /// Spawn a new thread and add it to the scheduler right away
    pub fn spawn<F>(fun: F)
        where F: Fn() -> ()
    {
        Scheduler::ready(Builder::new().spawn(fun));
    }
}

struct SchedulerImpl {
    // This is a little hack here. We are not allowed to have boxes on static
    // mutable variables. Instead we store the raw pointer stored in the Box.
    // We have to be extra careful to rebox it properly when needed,
    // Unique is used to represent ownership
    running: Option<ptr::Unique<ThreadImpl>>,
    ready_list: InterruptSpinLock<Queue<Box<ThreadImpl>, ThreadImpl>>,
}

impl SchedulerImpl {
    pub const fn new() -> Self {
        SchedulerImpl {
            running: None,
            ready_list: InterruptSpinLock::new(Queue::new()),
        }
    }

    /// Mark a thread as ready, and queue it to the ready queue of the
    /// scheduler
    pub fn ready(&mut self, mut thread: Thread) {
        thread.t_impl.state = State::Ready;

        self.ready_list.lock().enqueue(thread.t_impl);
    }

    pub unsafe fn schedule(&mut self) {
        loop {
            let (next, curr) = match self.running.take() {
                None => {
                    let n = self.ready_list
                                .lock()
                                .dequeue()
                                .and_then(|t| Some(ptr::Unique::new(Box::into_raw(t))));

                    (n, None)
                }
                Some(t) => {
                    let next = self.ready_list.lock().dequeue();

                    match next {
                        // We are the only thread that can be executed
                        None => {
                            self.running = Some(t);

                            break;
                        }
                        Some(n) => (Some(ptr::Unique::new(Box::into_raw(n))), Some(t)),
                    }
                }
            };

            match next {
                // No thread to execute => give control back to Xen and wait
                // for an interruption to unlock a thread
                None => ::xen::sched::block(),
                Some(mut n) => {
                    // XXX: Is there a better solution ?
                    let curr_ptr = curr.and_then(|mut c| {
                        let c_ptr: *mut _;

                        {
                            c_ptr = c.get_mut();
                        }

                        if c.get().state == State::Running {
                            c.get_mut().state = State::Ready;

                            self.ready_list
                                .lock()
                                .enqueue(Box::from_raw(c.get_mut()));
                        }

                        Some(c_ptr)
                    });

                    let next_ptr: *mut _;

                    {
                        next_ptr = n.get_mut();
                    }

                    n.get_mut().state = State::Running;

                    self.running = Some(n);

                    match curr_ptr {
                        None => ThreadImpl::empty().yield_to(&*next_ptr),
                        Some(p) => (*p).yield_to(&*next_ptr),
                    }

                    // Cleanup exited thread
                    if (*next_ptr).state == State::Terminated {
                        let b = Box::from_raw(next_ptr);

                        mem::drop(b);
                    }

                    // We break from the loop because we successfully yielded
                    // the CPU to some other thread and when we come back here
                    // we want to continue to execute the thread that
                    // originally called this function
                    break;
                }
            };
        }
    }

    pub unsafe fn terminate(&mut self) -> ! {
        if let Some(ref mut t) = self.running {
            t.get_mut().state = State::Terminated;
        }

        self.schedule();

        unreachable!()
    }
}
