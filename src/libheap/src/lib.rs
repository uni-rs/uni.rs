//! Implementation of various allocator algorithm:
//! - First Fit
//! - Buddy system

#![feature(no_std)]
#![feature(unique)]
#![feature(const_fn)]
#![feature(ptr_as_ref)]

#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;

extern crate intrusive;

use core::ptr;
use core::cmp;

macro_rules! align_up {
    ( $val:expr, $align:expr ) => {
        if ($val & ($align - 1)) == 0 {
            $val
        } else {
            ($val + $align - 1) & !($align - 1)
        }
    }
}

mod fit;
pub mod buddy;

pub use fit::FirstFit;
pub use buddy::Buddy;

/// Trait implemented by every allocator
pub trait Allocator {
    unsafe fn allocate(&mut self, size: usize, align: usize) -> *mut u8;
    unsafe fn deallocate(&mut self, ptr: *mut u8, old_size: usize, align: usize);

    unsafe fn reallocate(&mut self, ptr: *mut u8, old_size: usize, size: usize,
                         align: usize) -> *mut u8 {
        let new_ptr = self.allocate(size, align);

        if new_ptr.is_null() {
            return ptr::null_mut();
        }

        ptr::copy(ptr as *const u8, new_ptr, cmp::min(size, old_size));

        self.deallocate(ptr, old_size, align);

        new_ptr
    }
}
