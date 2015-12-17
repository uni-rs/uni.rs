//! Various spin lock definitions

use core::cell::UnsafeCell;

use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

use hal::xen::{disable_upcalls, set_upcalls_state};

pub use spin::Mutex as SpinLock;
pub use spin::MutexGuard as SpinGuard;

pub struct InterruptSpinLock<T> {
    data: UnsafeCell<T>,
    pub lock: AtomicBool,
}

pub struct InterruptSpinGuard<'a, T: 'a> {
    lock: &'a AtomicBool,
    data: &'a UnsafeCell<T>,
    upcalls_state: u8,
}

impl<T> InterruptSpinLock<T> {
    pub const fn new(data: T) -> Self {
        InterruptSpinLock {
            data: UnsafeCell::new(data),
            lock: AtomicBool::new(false),
        }
    }

    pub fn lock<'a>(&'a self) -> InterruptSpinGuard<'a, T> {
        let state = disable_upcalls();

        while self.lock.compare_and_swap(false, true, Ordering::SeqCst) {
        }

        InterruptSpinGuard {
            lock: &self.lock,
            data: &self.data,
            upcalls_state: state,
        }
    }
}

unsafe impl<T> Send for InterruptSpinLock<T> {}
unsafe impl<T> Sync for InterruptSpinLock<T> {}

impl<'a, T> Deref for InterruptSpinGuard<'a, T> {
    type Target = T;

    fn deref<'b>(&'b self) -> &'b T {
        unsafe { & *self.data.get() }
    }
}

impl<'a, T> DerefMut for InterruptSpinGuard<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<'a, T> Drop for InterruptSpinGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::SeqCst);
        set_upcalls_state(self.upcalls_state);
    }
}
