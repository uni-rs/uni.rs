//! An implementation of a buddy allocator.

use types::{UnsafeList, PhantomNode};

pub type FreeBlock = PhantomNode;

#[allow(dead_code)]
pub struct Buddy<'a> {
    min_block_size: usize,
    max_order: u32,
    free_lists: &'a mut [UnsafeList<FreeBlock>],
}
