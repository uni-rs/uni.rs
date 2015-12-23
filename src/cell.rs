//! Shareable mutable containers.

use core::ops::{Deref, DerefMut};

pub use core::cell::*;

/// Mutable memory location to wrap global variables
pub struct GlobalCell<T>(UnsafeCell<Option<T>>);

unsafe impl<T> Sync for GlobalCell<T> where T: Sync {}

impl<T> GlobalCell<T> {
    /// Create an empty new GlobalCell
    pub const fn new() -> Self {
        GlobalCell(UnsafeCell::new(None))
    }

    /// Set the contained value
    pub fn set(&self, value: T) {
        unsafe {
            *self.0.get() = Some(value);
        }
    }

    /// Get a reference over the wrapped value
    pub fn as_ref(&self) -> GlobalCellRef<T> {
        GlobalCellRef(&self.0)
    }

    /// Get a mutable reference over the wrapped value
    pub fn as_mut(&self) -> GlobalCellMutRef<T> {
        GlobalCellMutRef(&self.0)
    }
}

/// Represents a non mutable reference over the wrapped value of a GlobalCell<T>
pub struct GlobalCellRef<'a, T: 'a>(&'a UnsafeCell<Option<T>>);

/// Represents a mutable reference over the wrapped value of a GlobalCell<T>
pub struct GlobalCellMutRef<'a, T: 'a>(&'a UnsafeCell<Option<T>>);

impl<'a, T> Deref for GlobalCellRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            (*self.0.get()).as_ref().expect("Using uninitialized global cell")
        }
    }
}

impl<'a, T> Deref for GlobalCellMutRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            (*self.0.get()).as_ref().expect("Using uninitialized global cell")
        }
    }
}

impl<'a, T> DerefMut for GlobalCellMutRef<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            (*self.0.get()).as_mut().expect("Using uninitialized global cell")
        }
    }
}
