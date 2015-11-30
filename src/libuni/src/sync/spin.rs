//! Various spin lock definitions

use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

use xen::{disable_upcalls, set_upcalls_state};

pub use spin::Mutex as SpinLock;
pub use spin::MutexGuard as SpinGuard;

pub struct InterruptSpinLock<T> {
    data: T,
    lock: AtomicBool,
}

pub struct InterruptSpinGuard<'a, T: 'a> {
    lock: &'a mut AtomicBool,
    data: &'a mut T,
    upcalls_state: u8,
}

impl<T> InterruptSpinLock<T> {
    pub const fn new(data: T) -> Self {
        InterruptSpinLock {
            data: data,
            lock: AtomicBool::new(false),
        }
    }

    pub fn lock<'a>(&'a mut self) -> InterruptSpinGuard<'a, T> {
        let state = disable_upcalls();

        while self.lock.compare_and_swap(false, true, Ordering::SeqCst) {

        }

        InterruptSpinGuard {
            lock: &mut self.lock,
            data: &mut self.data,
            upcalls_state: state,
        }
    }
}

impl<'a, T> Deref for InterruptSpinGuard<'a, T> {
    type Target = T;

    fn deref<'b>(&'b self) -> &'b T {
        self.data
    }
}

impl<'a, T> DerefMut for InterruptSpinGuard<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        self.data
    }
}

impl<'a, T> Drop for InterruptSpinGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::SeqCst);
        set_upcalls_state(self.upcalls_state);
    }
}
