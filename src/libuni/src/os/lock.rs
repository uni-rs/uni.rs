use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct SpinLock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        SpinLock {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    fn do_lock(&self) {
        while self.lock.compare_and_swap(false, true, Ordering::SeqCst) {
            // spinlock
        }
    }

    pub fn lock(&self) -> SpinGuard<T> {
        self.do_lock();

        SpinGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }
}

pub struct SpinGuard<'a, T: 'a> {
    lock: &'a AtomicBool,
    data: &'a mut T,
}

impl<'a, T> Deref for SpinGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T> DerefMut for SpinGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T> Drop for SpinGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::SeqCst);
    }
}
