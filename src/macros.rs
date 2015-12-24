//! Definition of various macros

/// Print on the standard output
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::io::_print(format_args!($($arg)*));
    }
}

/// Print on the standard output with a new line
#[macro_export]
macro_rules! println {
    ($fmt:expr) => {
        print!(concat!($fmt, "\r\n"))
    };
    ($fmt:expr, $($arg:tt)*) => {
        print!(concat!($fmt, "\r\n"), $($arg)*)
    }
}

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

            $crate::thread::Scheduler::block(locked_queue);
        }
    )
}
