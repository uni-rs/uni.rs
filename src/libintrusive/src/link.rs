use core::ptr;
use core::mem;

#[derive(Debug)]
pub struct Link<T> {
    ptr: *mut T,
}

/// Non owner raw link base on *mut pointers
impl<T> Link<T> {
    pub fn none() -> Self {
        Link {
            ptr: ptr::null_mut(),
        }
    }

    pub fn some(ptr: *mut T) -> Self {
        Link {
            ptr: ptr,
        }
    }

    pub fn take(&mut self) -> Link<T> {
        mem::replace(self, Self::none())
    }

    pub fn as_ref(&self) -> Option<&T> {
        if self.ptr.is_null() {
            None
        } else {
            unsafe { Some(& *self.ptr) }
        }
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        if self.ptr.is_null() {
            None
        } else {
            unsafe { Some(&mut *self.ptr) }
        }
    }
}

impl<T> Clone for Link<T> {
    fn clone(&self) -> Self {
        Link {
            ptr: self.ptr,
        }
    }
}

impl<T> PartialEq for Link<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.ptr == rhs.ptr
    }
}
