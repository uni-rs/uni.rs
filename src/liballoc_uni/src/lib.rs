//! Integration crate to use heap::Buddy as Uni.rs allocator

#![feature(allocator)]
#![feature(const_fn)]

#![no_std]
#![allocator]

extern crate heap;
extern crate spin;

use heap::{Allocator, Buddy};
use heap::buddy::FreeList;

use spin::Mutex as SpinLock;

static mut ALLOCATOR: SpinLock<Option<Buddy<'static>>> = SpinLock::new(None);

const MIN_BLOCK_SIZE: usize = 16;

// 1MB
const MAX_ORDER: u32 = 16;

// XXX: Not pretty
static mut FREE_LISTS: [FreeList; (MAX_ORDER + 1) as usize] =
[
    FreeList::new(),
    FreeList::new(),
    FreeList::new(),
    FreeList::new(),
    FreeList::new(),
    FreeList::new(),
    FreeList::new(),
    FreeList::new(),
    FreeList::new(),
    FreeList::new(),
    FreeList::new(),
    FreeList::new(),
    FreeList::new(),
    FreeList::new(),
    FreeList::new(),
    FreeList::new(),
    FreeList::new()
];

pub unsafe fn init(region_start: usize, region_size: usize) {
    let mut guard = ALLOCATOR.lock();

    *guard = Some(Buddy::new(region_start, region_size, MIN_BLOCK_SIZE,
                             MAX_ORDER, &mut FREE_LISTS[..]));
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
