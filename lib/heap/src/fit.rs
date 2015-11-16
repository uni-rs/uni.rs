//! An implementation of a first fit allocator.
//!
//! The allocator uses a list of free blocks that can be allocated. When an
//! allocation is performed and the block is bigger than the requested size,
//! the block is splitted in 2 blocks: one used to fulfill the allocation, the
//! other is free.
//!
//! When a block is deallocated, the allocator tries to merge it with adjacent
//! blocks. The header of a block contains the necessary information to
//! retrieve the previous and next block if they exist. In order to access the
//! previous block, each block contains a footer that indicates the offset to
//! get the header.

use core::ptr;
use core::mem;

use super::types::Node;

const PREVIOUS_BIT: usize = 1;
const NEXT_BIT: usize = 2;
const SIZE_MASK: usize = PREVIOUS_BIT | NEXT_BIT;

/// The header is a meta data that is found before every block. It is use to
/// know if the block is available and its size. It also indicates if the
/// current block has a predecessor and a successor.
struct Header {
    size: usize,
    free: bool,
}

impl Header {
    pub fn new(size: usize, free: bool) -> Self {
        Header {
            size: size & !SIZE_MASK,
            free: free,
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.size & !SIZE_MASK
    }

    pub fn has_next(&self) -> bool {
        (self.size & NEXT_BIT) == NEXT_BIT
    }

    pub fn has_prev(&self) -> bool {
        (self.size & PREVIOUS_BIT) == PREVIOUS_BIT
    }

    pub fn set_next(&mut self) {
        self.size |= NEXT_BIT;
    }

    pub fn set_prev(&mut self) {
        self.size |= PREVIOUS_BIT;
    }

    pub unsafe fn get_next(&mut self) -> *mut Header {
        if !self.has_next() {
            return ptr::null_mut();
        }

        // A block is composed of a header, the data and a footer. If there is
        // a "next" block, it is going to be right after the footer of the
        // current block
        let mut ptr = self as *mut Self as *mut u8;

        ptr = ptr.offset(mem::size_of::<Header>() as isize);
        ptr = ptr.offset((self.size & !SIZE_MASK) as isize);
        ptr = ptr.offset(mem::size_of::<Footer>() as isize);

        ptr as *mut Header
    }

    pub unsafe fn get_prev(&mut self) -> *mut Header {
        if !self.has_prev() {
            return ptr::null_mut();
        }

        let mut ptr = self as *mut Self as *mut u8;

        ptr = ptr.offset(-(mem::size_of::<Footer>() as isize));

        let footer = ptr as *const Footer;

        ptr.offset(-((*footer).header_off as isize)) as *mut Header
    }
}

/// The footer is a meta data located after the block that helps the next block
/// to find the header of the current block.
struct Footer {
    header_off: usize,
}

/// A free block is linked with other free block. The extra space used does not
/// matter because the block is free.
type FreeBlock = Node<Header>;
