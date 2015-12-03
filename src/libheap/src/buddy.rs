//! An implementation of a buddy allocator.

use core::ptr;
use core::mem;
use core::cmp;

use Allocator;

use types::{UnsafeList, Link, PhantomNode};

pub type FreeBlock = PhantomNode;
pub type FreeList = UnsafeList<FreeBlock>;

#[allow(dead_code)]
pub struct Buddy<'a> {
    min_block_size: usize,
    min_block_order: u32,
    max_order: u32,
    heap_base: *mut u8,
    free_lists: &'a mut [FreeList]
}

impl<'a> Buddy<'a> {
    pub unsafe fn new(mut r_start: usize, r_size: usize, min_block_size: usize,
                      max_order: u32,
                      free_lists: &'a mut [FreeList]) -> Self {
        let max_block_size = min_block_size * 2usize.pow(max_order);
        let r_limit = r_start + r_size;

        assert!(min_block_size >= ::core::mem::size_of::<FreeBlock>());

        let mut ret = Buddy {
            min_block_size: min_block_size,
            min_block_order: Buddy::log2(min_block_size) as u32,
            max_order: max_order,
            heap_base: r_start as *mut u8,
            free_lists: free_lists,
        };

        // This algorithm only deals with max block at init, so return an empty
        // allocator if the region is not big enough
        if r_size < max_block_size {
            return ret;
        }

        while r_start < r_limit {
            ret.add_block(max_order, r_start as *mut u8);

            r_start += max_block_size;
        }

        ret
    }

    #[allow(exceeding_bitshifts)]
    pub fn next_power_of_two(mut num: usize) -> usize {
        if num == 0 {
            return 1;
        }

        num -= 1;

        num |= num >> 1;
        num |= num >> 2;
        num |= num >> 4;
        num |= num >> 8;
        num |= num >> 16;

        if mem::size_of::<usize>() == mem::size_of::<u64>() {
            num |= num >> 32;
        }

        num + 1
    }

    pub fn log2(mut num: usize) -> usize {
        let mut res = 0;

        loop {
            num >>= 1;

            if num == 0 {
                break;
            }

            res += 1;
        }

        res
    }

    unsafe fn add_block(&mut self, order: u32, start: *mut u8) {
        let link = Link::some(start as *mut FreeBlock);

        self.free_lists[order as usize].push_front(link);
    }

    unsafe fn split_block(&mut self, block: *mut u8, mut order: u32,
                          target_order: u32) {
        while order > target_order {
            order -= 1;

            let buddy_offset = self.get_size_from_order(order);
            let buddy_ptr = block.offset(buddy_offset as isize);

            self.add_block(order, buddy_ptr);
        }
    }

    unsafe fn find_and_pop_buddy(&mut self, ptr: *mut u8,
                                 order: u32) -> *mut u8{
        // Max order blocks are not merged
        if order == self.max_order {
            return ptr::null_mut();
        }

        let size = self.get_size_from_order(order);
        let ptr_from_base = ptr as usize - self.heap_base as usize;
        let buddy_ptr = self.heap_base.offset((ptr_from_base ^ size) as isize);
        let mut cursor = self.free_lists[order as usize].cursor();
        let mut found = false;

        loop {
            match cursor.as_ref() {
                None => break,
                Some(blk) => {
                    if blk as *const FreeBlock as *const u8 == buddy_ptr {
                        found = true;
                        break;
                    }
                }
            }

            cursor.next();
        }

        if found {
            cursor.remove();
            return buddy_ptr
        }

        ptr::null_mut()
    }

    fn get_order_from_size(&self, mut size: usize, _align: usize) -> u32 {
        size = Buddy::next_power_of_two(size);

        size = cmp::max(size, self.min_block_size);

        Buddy::log2(size) as u32 - self.min_block_order
    }

    fn get_size_from_order(&self, order: u32) -> usize {
        self.min_block_size * 2usize.pow(order)
    }
}

impl<'a> Allocator for Buddy<'a> {
    unsafe fn allocate(&mut self, size: usize, align: usize) -> *mut u8 {
        let order = self.get_order_from_size(size, align);

        if order > self.max_order {
            return ptr::null_mut();
        }

        for i in order..(self.max_order + 1) {
            let mut tmp = self.free_lists[i as usize].pop_front();

            if let Some(block) = tmp.as_mut() {
                let ptr = block as *mut FreeBlock as *mut u8;

                if i > order {
                    self.split_block(ptr, i, order);
                }

                return ptr
            }
        }

        ptr::null_mut()
    }

    unsafe fn deallocate(&mut self, mut ptr: *mut u8, old_size: usize,
                         align: usize) {
        let mut order = self.get_order_from_size(old_size, align);

        loop {
            let buddy = self.find_and_pop_buddy(ptr, order);

            if buddy == ptr::null_mut() {
                break;
            }

            ptr = cmp::min(ptr, buddy);

            order += 1;
        }

        self.add_block(order, ptr);
    }
}

#[cfg(test)]
mod test {
    use core::mem;
    use core::ptr;

    use Allocator;

    use super::{Buddy, FreeList};

    const HEAP_ALIGN: usize = 4096;
    const HEAP_SIZE: usize = 4096;

    // HEAP_ORDER = 5 & MIN_BLOCK_SIZE = 32 => max alloc 1024
    const HEAP_ORDER: u32 = 5;
    const MIN_BLOCK_SIZE: usize = 32;

    extern "C" {
        fn memalign(alignment: usize, size: usize) -> *mut u8;
        fn free(ptr: *mut u8);
    }

    #[test]
    fn test_buddy_next_power_of_two() {
        assert_eq!(Buddy::next_power_of_two(0), 1);
        assert_eq!(Buddy::next_power_of_two(2), 2);
        assert_eq!(Buddy::next_power_of_two(3), 4);
        assert_eq!(Buddy::next_power_of_two(5678), 8192);
        assert_eq!(Buddy::next_power_of_two(8192), 8192);
    }

    #[test]
    fn test_buddy_log2() {
        assert_eq!(Buddy::log2(0), 0);
        assert_eq!(Buddy::log2(1), 0);
        assert_eq!(Buddy::log2(2), 1);
        assert_eq!(Buddy::log2(4), 2);
        assert_eq!(Buddy::log2(8), 3);
        assert_eq!(Buddy::log2(16), 4);
        assert_eq!(Buddy::log2(0x87654321), 31);
    }

    #[test]
    fn test_buddy_alloc_dealloc() {
        unsafe {
            let heap = memalign(HEAP_ALIGN, HEAP_SIZE);
            let mut free_lists: [_; (HEAP_ORDER + 1) as usize];

            free_lists = mem::uninitialized();

            for i in 0..(HEAP_ORDER + 1) {
                free_lists[i as usize] = FreeList::new();
            }

            let max_size = MIN_BLOCK_SIZE * 2usize.pow(HEAP_ORDER);

            let mut alloc = Buddy::new(heap as usize, HEAP_SIZE, MIN_BLOCK_SIZE,
                                       HEAP_ORDER, &mut free_lists[..]);

            // Allocation is too big
            assert_eq!(alloc.allocate(max_size + 1, 1), ptr::null_mut());

            {
                let max_blk = alloc.allocate(max_size, 1);
                let last_blk_offset = ((HEAP_SIZE / max_size) - 1) * max_size;

                // Due to Buddy::new using push front, the first allocated block
                // is gonna be the last pushed
                assert_eq!(max_blk, heap.offset(last_blk_offset as isize));

                alloc.deallocate(max_blk, max_size, 1);
            }

            let blk_32_1 = alloc.allocate(32, 1);
            let blk_32_2 = alloc.allocate(32, 1);

            assert_eq!(blk_32_1.offset(32), blk_32_2);

            let blk_64_1 = alloc.allocate(64, 1);

            assert_eq!(blk_32_1.offset(64), blk_64_1);

            alloc.deallocate(blk_32_1, 32, 1);
            alloc.deallocate(blk_32_2, 32, 1);

            let blk_32_1_1 = alloc.allocate(32, 1);
            let blk_32_2_1 = alloc.allocate(32, 1);

            assert_eq!(blk_32_1_1, blk_32_1);
            assert_eq!(blk_32_2_1, blk_32_2);

            alloc.deallocate(blk_32_2_1, 32, 1);
            alloc.deallocate(blk_32_1_1, 32, 1);

            let blk_64_2 = alloc.allocate(64, 1);

            assert_eq!(blk_64_2, blk_32_1);

            let blk_128 = alloc.allocate(128, 1);

            assert_eq!(blk_128, blk_64_1.offset(64));

            alloc.deallocate(blk_64_1, 64, 1);
            alloc.deallocate(blk_64_2, 64, 1);
            alloc.deallocate(blk_128, 128, 1);

            free(heap);
        }
    }
}
