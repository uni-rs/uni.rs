//! Integration crate to use heap::FirstFit as Uni.rs allocator

#![feature(no_std)]
#![feature(allocator)]
#![feature(const_fn)]

#![no_std]
#![allocator]

extern crate heap;
extern crate spin;

use heap::{Allocator, FirstFit};

use spin::Mutex as SpinLock;

static mut ALLOCATOR: SpinLock<Option<FirstFit>> = SpinLock::new(None);

pub unsafe fn init(region_start: usize, region_size: usize) {
    let mut guard = ALLOCATOR.lock();

    *guard = Some(FirstFit::new(region_start as *mut u8, region_size));
}

#[no_mangle]
pub extern "C" fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    unsafe {
        ALLOCATOR.lock().as_mut()
            .expect("Use of uninitialized allocator").allocate(size, align)
    }
}

#[no_mangle]
pub extern "C" fn __rust_deallocate(ptr: *mut u8, old_size: usize,
                                    align: usize) {
    unsafe {
        ALLOCATOR.lock().as_mut()
            .expect("Use of uninitialized allocator")
            .deallocate(ptr, old_size, align);
    }
}

#[no_mangle]
pub extern "C" fn __rust_reallocate(ptr: *mut u8, old_size: usize, size: usize,
                                    align: usize) -> *mut u8 {
    unsafe {
        ALLOCATOR.lock().as_mut()
            .expect("Use of uninitialized allocator")
            .reallocate(ptr, old_size, size, align)
    }
}

#[no_mangle]
pub extern "C" fn __rust_reallocate_inplace(_: *mut u8, old_size: usize,
                                            _: usize, _: usize) -> usize
{
    old_size
}

#[no_mangle]
pub extern "C" fn __rust_usable_size(size: usize, _: usize) -> usize {
    size
}
