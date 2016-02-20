//! Various spin lock definitions

use core::cell::UnsafeCell;

use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

use hal::{local_irq_save, local_irq_restore};

pub use spin::Mutex as SpinLock;
pub use spin::MutexGuard as SpinGuard;

pub use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct InterruptSpinLock<T> {
    data: UnsafeCell<T>,
    pub lock: AtomicBool,
}

pub struct InterruptSpinGuard<'a, T: 'a> {
    lock: &'a AtomicBool,
    data: &'a UnsafeCell<T>,
    irq_state: usize,
}

impl<T> InterruptSpinLock<T> {
    pub const fn new(data: T) -> Self {
        InterruptSpinLock {
            data: UnsafeCell::new(data),
            lock: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) -> InterruptSpinGuard<T> {
        let state = local_irq_save();

        while self.lock.compare_and_swap(false, true, Ordering::SeqCst) {
        }

        InterruptSpinGuard {
            lock: &self.lock,
            data: &self.data,
            irq_state: state,
        }
    }
}

unsafe impl<T> Send for InterruptSpinLock<T> {}
unsafe impl<T> Sync for InterruptSpinLock<T> {}

impl<'a, T> Deref for InterruptSpinGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { & *self.data.get() }
    }
}

impl<'a, T> DerefMut for InterruptSpinGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<'a, T> Drop for InterruptSpinGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::SeqCst);
        local_irq_restore(self.irq_state);
    }
}
