use alloc::boxed::Box;

use sync::spin::InterruptSpinLock;
use intrusive::queue::Queue;

use super::{Thread, ThreadImpl, Scheduler};

pub type InternalQueue = Queue<Box<ThreadImpl>, ThreadImpl>;

pub struct WaitQueue {
    queue: InterruptSpinLock<InternalQueue>,
}

impl WaitQueue {
    /// Create a new WaitQueue
    pub fn new() -> Self {
        WaitQueue {
            queue: InterruptSpinLock::new(Queue::new()),
        }
    }

    #[inline]
    /// Block the current thread
    pub fn block(&self) {
        ::hal::xen::disable_upcalls();
        Scheduler::block(self.queue.lock());
    }

    #[inline]
    fn unblock_thread(imp: Box<ThreadImpl>) {
        let thread = Thread {
            t_impl: imp,
        };

        Scheduler::ready(thread);
    }

    /// Unblock the first thread in the queue.
    /// Returns true if a thread was unblocked, false otherwise
    pub fn unblock(&self) {
        if let Some(t) = self.queue.lock().dequeue() {
            WaitQueue::unblock_thread(t);
        }
    }

    /// Unblock all threads in the queue
    pub fn unblock_all(&self) {
        let mut queue = self.queue.lock();

        while let Some(t) = queue.dequeue() {
            WaitQueue::unblock_thread(t);
        }
    }
}

unsafe impl Send for WaitQueue {}
unsafe impl Sync for WaitQueue {}
