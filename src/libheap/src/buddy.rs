//! An implementation of a buddy allocator.

use core::ptr;

use Allocator;

use types::{UnsafeList, PhantomNode};

pub type FreeBlock = PhantomNode;

#[allow(dead_code)]
pub struct Buddy<'a> {
    min_block_size: usize,
    max_order: u32,
    free_lists: &'a mut [UnsafeList<FreeBlock>],
}

impl<'a> Allocator for Buddy<'a> {
    unsafe fn allocate(&mut self, mut _size: usize, _align: usize) -> *mut u8 {
        ptr::null_mut()
    }

    unsafe fn deallocate(&mut self, _ptr: *mut u8, _old_size: usize,
                         _align: usize) {

    }
}
