use alloc::boxed::Box;

use intrusive::queue::Queue;

use hal::local_irq_disable;

use sync::spin::{InterruptSpinLock, InterruptSpinGuard};

use super::{Thread, ThreadImpl, Scheduler};

pub type InternalQueue = Queue<Box<ThreadImpl>, ThreadImpl>;

#[macro_export]
/// Wait for an event to occur
///
/// This macro allows a thread to wait for a condition to become true.
/// If the condition is false, the thread will block on the [`queue`][queue]
/// given as parameter. This macro *MUST* be called with local irqs enabled.
///
/// [queue]: thread/struct.WaitQueue.html
///
/// Note that this macro does not signify exclusivity. In fact multiple
/// concurrent threads might go through. In that case external atomic (or
/// locked) test on the condition might be necessary.
macro_rules! wait_event {
    ($queue:expr, $cond:expr) => (
        loop {
            $crate::hal::local_irq_disable();

            let locked_queue = $queue.lock();

            if $cond {
                $crate::hal::local_irq_enable();
                break;
            }

            Scheduler::block(locked_queue);
        }
    )
}

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

    #[doc(hidden)]
    pub fn lock(&self) -> InterruptSpinGuard<InternalQueue> {
        self.queue.lock()
    }

    #[inline]
    /// Block the current thread
    pub fn block(&self) {
        local_irq_disable();
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
