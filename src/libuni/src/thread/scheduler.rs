//! Scheduler used in Uni.rs

use core::ptr;
use core::mem;

use hal::{local_irq_disable, local_irq_enable};

use alloc::boxed::Box;

use sync::spin::{InterruptSpinLock, InterruptSpinGuard};
use intrusive::queue::Queue;

use super::{Thread, Builder, ThreadImpl, State};
use super::wait_queue::InternalQueue;

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
            SCHEDULER.schedule(true);
        }
    }

    #[doc(hidden)]
    /// Block the current thread inside a wait queue.
    pub fn block<'a>(queue: InterruptSpinGuard<'a, InternalQueue>) {
        unsafe {
            SCHEDULER.block(queue);
        }
    }

    #[doc(hidden)]
    /// Terminate the execution of the current thread
    pub fn terminate() -> ! {
        unsafe {
            SCHEDULER.terminate();
        }
    }

    /// Spawn a new thread and add it to the scheduler right away
    pub fn spawn<F>(fun: F)
        where F: FnMut() -> (), F: Send + 'static
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
    cleanup_queue: InterruptSpinLock<Queue<Box<ThreadImpl>, ThreadImpl>>,
}

impl SchedulerImpl {
    pub const fn new() -> Self {
        SchedulerImpl {
            running: None,
            ready_list: InterruptSpinLock::new(Queue::new()),
            cleanup_queue: InterruptSpinLock::new(Queue::new()),
        }
    }

    /// Mark a thread as ready, and queue it to the ready queue of the
    /// scheduler
    pub fn ready(&self, mut thread: Thread) {
        thread.t_impl.state = State::Ready;

        self.ready_list.lock().enqueue(thread.t_impl);
    }

    unsafe fn cleanup(&mut self) {
        let mut queue = self.cleanup_queue.lock();

        while let Some(t) = queue.dequeue() {
            mem::drop(t);
        }
    }

    // Return the next thread to run. If None is returned then it means that no
    // thread is ready to be scheduled and the current thread was just blocked
    // or terminated
    unsafe fn elect_next(&mut self) -> Option<ptr::Unique<ThreadImpl>> {
        match self.running.take() {
            None => self.ready_list
                        .lock()
                        .dequeue()
                        .and_then(|t| Some(ptr::Unique::new(Box::into_raw(t)))),
            Some(mut r) => {
                let next = self.ready_list.lock().dequeue();

                match next {
                    None => {
                        if r.get().state == State::Running {
                            self.running = Some(r);
                        }

                        None
                    }
                    Some(n) => {
                        if r.get().state == State::Running {
                            r.get_mut().state = State::Ready;

                            self.ready_list
                                .lock()
                                .enqueue(Box::from_raw(r.get_mut()));
                        }

                        Some(ptr::Unique::new(Box::into_raw(n)))
                    }
                }
            }
        }
    }

    unsafe fn switch_to_next(&mut self, mut t: ptr::Unique<ThreadImpl>) -> ! {
        let t_ptr: *const _;

        {
            t_ptr = t.get();
        }

        t.get_mut().state = State::Running;

        self.running = Some(t);

        // Called with interruptions disabled
        local_irq_enable();

        (*t_ptr).context.load();
    }

    pub unsafe fn schedule(&mut self, cleanup: bool) {
        if cleanup {
            // Properly cleanup terminated threads
            self.cleanup();
        }

        if let Some(ref mut r) = self.running {
            // As explained in Context::save() this call will return twice.
            // When this function returns true that means that the thread was
            // just restored and shall continue its execution. This is why we
            // immediately return
            if r.get_mut().context.save() {
                return;
            }
        }

        loop {
            local_irq_disable();

            let next = self.elect_next();

            // The only time elect_next() leaves self.running not None is when
            // the current thread is running and there is no other thread ready
            // to be scheduled. In this case let the current thread continue
            if self.running.is_some() {
                local_irq_enable();
                break;
            }

            match next {
                None => {
                    // If no threads can be run wait for an interruption. An
                    // interruption might wake up a thread for us to run
                    ::hal::xen::sched::block();
                    local_irq_enable();
                }
                Some(next) => {
                    self.switch_to_next(next);
                }
            }
        }
    }

    pub unsafe fn block<'a>(&mut self,
                            mut queue: InterruptSpinGuard<'a, InternalQueue>) {
        if let Some(ref mut r) = self.running {
            r.get_mut().state = State::Blocked;

            queue.enqueue(Box::from_raw(r.get_mut()));

            if r.get_mut().context.save() {
                // We forget the queue because it was dropped when the function
                // was actually blocking the thread. If we are here that means
                // that we were just unblocked (for more info see comment in
                // schedule())
                mem::forget(queue);

                return;
            }
        }

        let next = self.elect_next();

        // The spinlock guard is taken as parameter to avoid races on multiple
        // CPUs (even if not supported ATM). The queue is released here because
        // it is safe to use the current thread (that was just blocked) in an
        // other CPU. Indeed its context was just saved and it was removed from
        // being self.current. The current CPU has no more reference to that
        // thread so its safe to be unblocked by another CPU.
        mem::drop(queue);

        match next {
            None => {
                // See comment in schedule()
                ::hal::xen::sched::block();
                local_irq_enable();
            }
            Some(next) => {
                self.switch_to_next(next);
            }
        }

        // We don't call schedule before because we need to release that lock
        // after elect_next() runs to remove all references to the blocked
        // thread in the current CPU
        self.schedule(true);
    }

    pub unsafe fn terminate(&mut self) -> ! {
        if let Some(ref mut t) = self.running {
            t.get_mut().state = State::Terminated;

            self.cleanup_queue.lock().enqueue(Box::from_raw(t.get_mut()));
        }

        // Don't cleanup terminated threads as this would mean that we release
        // the current thread, which would lead to bad things if done while
        // its still running.
        self.schedule(false);

        unreachable!()
    }
}
