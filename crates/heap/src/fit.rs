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
use core::cmp;

use intrusive::link::Link;
use intrusive::list::{List, Node};

use super::Allocator;

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
type FreeBlock = DataNode<Header>;

pub struct FirstFit {
    min_align: usize,
    free_blocks: List<ptr::Unique<FreeBlock>, FreeBlock>,
}

impl FirstFit {
    pub unsafe fn new(r_start: *mut u8, mut r_size: usize) -> Self {
        let mut alloc = FirstFit {
            // The minimum size of a block is the size of 2 pointers in order
            // to store the free list links when the block is free
            min_align: mem::size_of::<FreeBlock>() - mem::size_of::<Header>(),
            free_blocks: List::new(),
        };

        r_size -= mem::size_of::<Header>() + mem::size_of::<Footer>();

        let blk_link = FirstFit::create_block(r_start, r_size, true);

        alloc.free_blocks.push_front(blk_link);

        alloc
    }

    unsafe fn create_block(start: *mut u8, size: usize,
                           free: bool) -> ptr::Unique<FreeBlock> {
        let blk = start as *mut FreeBlock;

        let footer_off = size + mem::size_of::<Header>();
        let footer = start.offset(footer_off as isize) as *mut Footer;

        *blk = FreeBlock::new(Header::new(size, free));

        (*footer).header_off = footer as usize - start as usize;

        ptr::Unique::new(blk)
    }

    // Split the blk block given in parameter in 2 blocks:
    // - One to serve the allocation of size "size" (b1)
    // - One to keep the remaining free data (b2)
    //
    // Before split:
    //
    // +------------+------------------------------------------+-----------+
    // | Hdr free=1 |                   DATA                   |    Foot   |
    // +------------+------------------------------------------+-----------+
    //
    // After split:
    //
    // +------------+-------------+-----------+-----------+----+-----------+
    // | Hdr free=0 | DATA (size) |    Foot   | Hdr free=1|DATA|   Foot    |
    // +------------+-------------+-----------+-----------+----+-----------+
    // |-----------------> b1 <---------------|-----------> b2 <-----------|
    //
    unsafe fn split_block(&mut self, blk: &mut FreeBlock, size: usize) -> *mut u8 {
        let blk_ptr = blk as *mut FreeBlock as *mut u8;
        let res = blk_ptr.offset(mem::size_of::<Header>() as isize);

        let min_size;

        // The minimum size of a block is self.min_align, but the block
        // also has to contain the metadata (header + footer)
        min_size = size + mem::size_of::<Header>() + mem::size_of::<Footer>() +
                   self.min_align;

        // The block is not large enough to be splitted
        if blk.data().size() < min_size {
            blk.data_mut().free = false;

            return res
        }

        {
            // Craft the free block (b2)
            let free_offset = size + mem::size_of::<Header>() +
                              mem::size_of::<Footer>();
            let free_size = blk.data().size() - free_offset;
            let free_blk_ptr = blk_ptr.offset(free_offset as isize);

            let free_blk = FirstFit::create_block(free_blk_ptr, free_size, true);
            let free_blk_mut = free_blk.as_mut().unwrap();

            // Previous block is b1
            free_blk_mut.data_mut().set_prev();

            if blk.data().has_next() {
                free_blk_mut.data_mut().set_next();
            }

            self.free_blocks.push_front(free_blk);
        }

        {
            // Craft the allocated block (b1)
            // We just need to craft it, it is not added in any list. Useful
            // information will be retrieved from the header when the block
            // is freed
            let has_prev = blk.data_mut().has_prev();

            FirstFit::create_block(blk_ptr, size, false);

            if has_prev {
                blk.data_mut().set_prev();
            }

            // Next block is b2
            blk.data_mut().set_next();
        }

        res
    }

    unsafe fn remove_from_free_list(&mut self, block: *mut FreeBlock) {
        let mut list_cursor = self.free_blocks.cursor();

        loop {
            if let Some(elem) = list_cursor.next_peek() {
                if elem as *mut FreeBlock == block {
                    break;
                }
            } else {
                break;
            }

            list_cursor.next();
        }

        list_cursor.remove();
    }

    unsafe fn merge_with_prev(&mut self, hdr: &mut Header) -> *mut Header {
        let mut res = hdr as *mut Header;

        if hdr.has_prev() {
            let prev = hdr.get_prev();

            if (*prev).free {
                let has_prev = (*prev).has_prev();
                let has_next = hdr.has_next();
                let new_size = hdr.size() + (*prev).size() +
                               mem::size_of::<Header>() +
                               mem::size_of::<Footer>();

                self.remove_from_free_list(prev as *mut FreeBlock);

                // We don't care about the result of the call because we
                // already know the beginning of the block (prev)
                FirstFit::create_block(prev as *mut u8, new_size, true);

                if has_prev {
                    (*prev).set_prev();
                }

                if has_next {
                    (*prev).set_next();
                }

                res = prev;
            }
        }

        res
    }

    unsafe fn merge_with_next(&mut self, hdr: &mut Header) {
        if hdr.has_next() {
            let next = hdr.get_next();

            if (*next).free {
                let has_prev = hdr.has_prev();
                let has_next = (*next).has_next();
                let next_ref = next.as_mut().unwrap();
                let new_size = hdr.size() + next_ref.size() +
                               mem::size_of::<Header>() +
                               mem::size_of::<Footer>();
                let hdr_ptr = hdr as *mut Header;

                self.remove_from_free_list(hdr_ptr as *mut FreeBlock);

                // We don't care about the result of the call because we
                // already know the beginning of the block (hdr)
                FirstFit::create_block(hdr_ptr as *mut u8, new_size, true);

                if has_prev {
                    hdr.set_prev();
                }

                if has_next {
                    hdr.set_next();
                }
            }
        }
    }

    unsafe fn merge(&mut self, mut hdr: *mut Header) -> ptr::Unique<FreeBlock> {
        // Try to merge with previous block if it exists and is free.
        hdr = self.merge_with_prev(hdr.as_mut().unwrap());

        // Try to merge with next if it exists and is free
        self.merge_with_next(hdr.as_mut().unwrap());

        ptr::Unique::new(hdr as *mut FreeBlock)
    }
}

impl Allocator for FirstFit {
    unsafe fn allocate(&mut self, mut size: usize, align: usize) -> *mut u8 {
        let mut blk_link;

        size = align_up!(size, cmp::max(align, self.min_align));

        {
            let mut cursor = self.free_blocks.cursor();

            loop {
                match cursor.next_peek() {
                    None => break,
                    Some(node) => {
                        if node.data().size() >= size {
                            break;
                        }
                    }
                }

                cursor.next();
            }

            blk_link = cursor.remove();
        }

        match blk_link.as_mut() {
            None => ptr::null_mut(),
            Some(blk) => self.split_block(blk.get_mut(), size),
        }
    }

    unsafe fn deallocate(&mut self, ptr: *mut u8, _: usize, _: usize) {
        if ptr == ptr::null_mut() {
            return;
        }

        let hdr = (ptr as *mut Header).offset(-1);

        (*hdr).free = true;

        let node = self.merge(hdr);

        self.free_blocks.push_front(node);
    }
}

pub struct DataNode<T> {
    elem: T,
    prev: Link<DataNode<T>>,
    next: Link<DataNode<T>>,
}

impl<T> DataNode<T> {
    pub fn new(elem: T) -> Self {
        DataNode {
            elem: elem,
            prev: Link::none(),
            next: Link::none(),
        }
    }

    pub fn data(&self) -> &T {
        &self.elem
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.elem
    }
}

impl<T> Node for DataNode<T> {
    fn prev(&self) -> &Link<DataNode<T>> {
        &self.prev
    }

    fn next(&self) -> &Link<DataNode<T>> {
        &self.next
    }

    fn prev_mut(&mut self) -> &mut Link<DataNode<T>> {
        &mut self.prev
    }

    fn next_mut(&mut self) -> &mut Link<DataNode<T>> {
        &mut self.next
    }
}

#[cfg(test)]
mod test {
    use super::FirstFit;
    use super::{Header, Footer, FreeBlock};
    use super::super::Allocator;

    use core::mem::size_of;
    use core::ptr::null_mut;
    use core::ptr::write_bytes;

    use intrusive::list::Node;

    const HEAP_SIZE: usize = 4096;
    const HEAP_ALIGN: usize = 16;

    extern "C" {
        fn memalign(alignment: usize, size: usize) -> *mut u8;
        fn free(ptr: *mut u8);
    }

    unsafe fn create_allocator(size: usize) -> (FirstFit, *mut u8) {
        let heap = memalign(4096, size);
        let allocator = FirstFit::new(heap, size);

        let actual_size = size - size_of::<Header>() - size_of::<Footer>();

        // Verify the init block
        let init_block = (heap as *const FreeBlock).as_ref().unwrap();

        assert!(init_block.data().free);
        assert!(!init_block.data().has_next());
        assert!(!init_block.data().has_prev());
        assert!(init_block.next().as_ref().is_none());
        assert!(init_block.prev().as_ref().is_none());
        assert_eq!(init_block.data().size, actual_size);

        (allocator, heap)
    }

    #[test]
    fn test_new_first_fit() {
        let vec = vec![32, 1024, 4096, 4194304];

        for size in vec {
            unsafe {
                let (_, heap) = create_allocator(size);

                free(heap);
            }
        }
    }

    unsafe fn alloc_test(allocator: &mut FirstFit,
                         size: usize) -> *const Header {
        let real_size = align_up!(size, HEAP_ALIGN);
        let ptr = allocator.allocate(size, HEAP_ALIGN);

        assert!(ptr != null_mut());

        let hdr = ptr.offset(-(size_of::<Header>() as isize)) as *const Header;

        // Be careful when calling this function because this test might fail
        // if the block that the allocator is gonna use cannot be splitted
        assert_eq!((*hdr).size(), real_size);
        assert!(!(*hdr).free);

        hdr
    }

    #[test]
    fn test_alloc_dealloc_first_fit() {
        unsafe {
            let (mut allocator, heap) = create_allocator(HEAP_SIZE);

            assert_eq!(allocator.allocate(HEAP_SIZE + 1, HEAP_ALIGN),
                       null_mut());

            let blk1 = alloc_test(&mut allocator, 10);
            let blk2 = alloc_test(&mut allocator, 100);
            let blk3 = alloc_test(&mut allocator, 231);

            allocator.deallocate(blk2.offset(1) as *mut u8, 100, HEAP_ALIGN);

            assert!((*blk2).free);

            let blk4 = alloc_test(&mut allocator, 100);

            assert_eq!(blk2, blk4);

            allocator.deallocate(blk3.offset(1) as *mut u8, 231, HEAP_ALIGN);
            allocator.deallocate(blk1.offset(1) as *mut u8, 10, HEAP_ALIGN);
            allocator.deallocate(blk4.offset(1) as *mut u8, 100, HEAP_ALIGN);

            assert!(allocator.allocate(HEAP_SIZE - size_of::<Header>() -
                                       size_of::<Footer>() - 500, HEAP_ALIGN) !=
                    null_mut());

            free(heap);
        }
    }

    #[test]
    fn test_realloc_first_fit() {
        unsafe {
            let (mut allocator, heap) = create_allocator(HEAP_SIZE);

            let mut ptr = allocator.allocate(5, 1);

            write_bytes(ptr, 33, 5);

            ptr = allocator.reallocate(ptr, 5, 10, 1);

            write_bytes(ptr, 34, 10);

            ptr = allocator.reallocate(ptr, 10, 20, 1);

            write_bytes(ptr, 35, 20);

            free(heap);
        }
    }
}
