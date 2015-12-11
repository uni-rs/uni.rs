//! Wrapper around stack creation and management.

use core::cmp;

use alloc_uni::{__rust_allocate, __rust_deallocate};

const MIN_STACK_SIZE: usize = 4096;

pub struct Stack {
    ptr: Option<*mut u8>,
    size: usize,
}

impl Stack {
    pub fn null() -> Self {
        Stack {
            ptr: None,
            size: 0,
        }
    }

    // TODO: Verify that size is a power of 2. While this is not enforced
    // this method is unsafe
    pub unsafe fn new(size: usize) -> Self {
        let stack_size = cmp::max(size, MIN_STACK_SIZE);

        // We use a buddy allocator so we know the stack size is gonna be a
        // power of two
        let stack_ptr = __rust_allocate(stack_size, 4096);

        if stack_ptr.is_null() {
            Stack::null()
        } else {
            Stack {
                ptr: Some(stack_ptr),
                size: stack_size,
            }
        }
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.ptr.is_none()
    }

    pub unsafe fn top(&self) -> Option<*mut u8> {
        self.ptr.and_then(|ptr| {
            Some(ptr.offset(self.size as isize))
        })
    }

    #[allow(dead_code)]
    pub unsafe fn bottom(&self) -> Option<*mut u8> {
        self.ptr
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        if let Some(ptr) = self.ptr {
            __rust_deallocate(ptr, self.size, 4096);
        }
    }
}
