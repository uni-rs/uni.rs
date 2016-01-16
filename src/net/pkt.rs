//! Network packet utility

use alloc_uni::__rust_deallocate;

use hal::arch::defs::PAGE_SIZE;

/// A network packet
pub struct Packet {
    page: *mut u8,
    data: *mut u8,
    size: usize,
}

impl Packet {
    /// Creates a new packet from a memory `page`
    ///
    /// `offset` is the offset from the beginning of the page where the actual
    /// network data is
    ///
    /// `size` is the size in byte of the packet
    ///
    /// This method is unsafe because a few requirements are necessary:
    ///
    /// * `page` must be allocated with `__rust_allocate(PAGE_SIZE, PAGE_SIZE)`
    /// * `page` ownership is transferred to this packet (i.e. it will be
    ///   deallocated on `Drop`)
    pub unsafe fn new(page: *mut u8, offset: usize, size: usize) -> Self {
        Packet {
            page: page,
            data: page.offset(offset as isize),
            size: size,
        }
    }

    #[inline]
    /// Returns the size of the packet
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline]
    /// Returns the offset from `page()` where the data starts
    pub fn offset(&self) -> usize {
        self.data as usize - self.page as usize
    }

    #[inline]
    /// Returns the beginning of the page allocated
    pub fn page(&self) -> *const u8 {
        self.page
    }

    #[inline]
    /// Returns a pointer that points to the beginning of the network data
    pub fn data(&self) -> *const u8 {
        self.data
    }
}

impl Drop for Packet {
    fn drop(&mut self) {
        __rust_deallocate(self.page, PAGE_SIZE, PAGE_SIZE);
    }
}
