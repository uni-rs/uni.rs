//! An implementation of a buddy allocator.

use core::ptr;
use core::mem;

use Allocator;

use types::{UnsafeList, PhantomNode};

pub type FreeBlock = PhantomNode;

#[allow(dead_code)]
pub struct Buddy<'a> {
    min_block_size: usize,
    max_order: u32,
    free_lists: &'a mut [UnsafeList<FreeBlock>],
}

impl<'a> Buddy<'a> {
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
}

impl<'a> Allocator for Buddy<'a> {
    unsafe fn allocate(&mut self, mut _size: usize, _align: usize) -> *mut u8 {
        ptr::null_mut()
    }

    unsafe fn deallocate(&mut self, _ptr: *mut u8, _old_size: usize,
                         _align: usize) {

    }
}

#[cfg(test)]
mod test {
    use super::Buddy;

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
}
