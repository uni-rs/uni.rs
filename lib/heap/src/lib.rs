#![feature(no_std)]
#![feature(ptr_as_ref)]

#![no_std]

use core::ptr;
use core::cmp;

mod fit;

mod types;

pub trait Allocator {
    unsafe fn allocate(&mut self, size: usize, align: usize) -> *mut u8;
    unsafe fn deallocate(&mut self, ptr: *mut u8, old_size: usize, align: usize);

    unsafe fn reallocate(&mut self, ptr: *mut u8, old_size: usize, size: usize,
                  align: usize) -> *mut u8 {
        let new_ptr = self.allocate(size, align);

        if new_ptr.is_null() {
            return ptr::null_mut();
        }

        ptr::copy(ptr, new_ptr, cmp::min(size, old_size));

        self.deallocate(ptr, old_size, align);

        new_ptr
    }
}
