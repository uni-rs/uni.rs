use core::ptr;
use core::cell::Cell;

use alloc::boxed::Box;

use sync::Arc;

use sync::spin::InterruptSpinLock;

/// Inspired by std::io::lazy::Lazy
pub struct Lazy<T> {
    lock: InterruptSpinLock<()>,
    ptr: Cell<*mut Arc<T>>,
    init: fn() -> Arc<T>,
}

unsafe impl<T> Sync for Lazy<T> {}

impl<T> Lazy<T> {
    pub const fn new(init: fn() -> Arc<T>) -> Self {
        Lazy {
            lock: InterruptSpinLock::new(()),
            ptr: Cell::new(ptr::null_mut()),
            init: init,
        }
    }

    pub unsafe fn get(&'static self) -> Arc<T> {
        let _lock = self.lock.lock();

        if self.ptr.get().is_null() {
            self.init()
        } else {
            (*self.ptr.get()).clone()
        }
    }

    pub unsafe fn init(&'static self) -> Arc<T> {
        let arc = (self.init)();

        self.ptr.set(Box::into_raw(Box::new(arc.clone())));

        arc
    }
}
