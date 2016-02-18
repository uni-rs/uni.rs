//! Network packet utility

use core::{mem, slice};

use net::{Interface, InterfaceWeak};

use alloc_uni::__rust_deallocate;

use hal::arch::defs::PAGE_SIZE;

/// A network packet
pub struct Packet {
    /// The page that contains the packet. This is aligned on PAGE_SIZE and
    /// must be allocated by __rust_allocate(PAGE_SIZE, PAGE_SIZE)
    page: *mut u8,
    /// Pointer to the start of the data.
    data: *mut u8,
    /// Size of the packet
    size: usize,
    /// The interface the packet was received on
    intf: Option<InterfaceWeak>,
    /// Size of the link header
    link_hdr_size: usize,
    /// Size of the network header
    net_hdr_size: usize,
    /// Size of the transport header
    tspt_hdr_size: usize,
}

impl Packet {
    /// Creates a new packet from a memory pointer `page`
    ///
    /// `offset` is the offset from the beginning of the page where the actual
    /// network data is
    ///
    /// `size` is the size in bytes of the packet
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
            intf: None,
            link_hdr_size: 0,
            net_hdr_size: 0,
            tspt_hdr_size: 0,
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
    /// Returns the beginning of the allocated page
    pub fn page(&self) -> *const u8 {
        self.page
    }

    #[inline]
    /// Returns a pointer that points to the beginning of the network data
    pub fn start(&self) -> *const u8 {
        self.data
    }

    #[inline]
    /// Return the interface the packet is linked to
    pub fn interface(&self) -> Option<Interface> {
        self.intf.as_ref().and_then(|i| i.upgrade())
    }

    #[inline]
    /// Set the interface the packet is linked to
    pub fn set_interface(&mut self, intf: InterfaceWeak) {
        self.intf = Some(intf);
    }

    #[inline]
    /// Returns the size of the link header
    ///
    /// This has to be set by the link layer handler
    pub fn link_hdr_size(&self) -> usize {
        self.link_hdr_size
    }

    #[inline]
    /// Returns a mutable reference to the size of the link header
    pub unsafe fn link_hdr_size_mut(&mut self) -> &mut usize {
        &mut self.link_hdr_size
    }

    #[inline]
    /// Returns the size of the network header
    ///
    /// This has to be set by the network layer handler
    pub fn net_hdr_size(&self) -> usize {
        self.net_hdr_size
    }

    #[inline]
    /// Returns a mutable reference to the size of the network header
    pub unsafe fn net_hdr_size_mut(&mut self) -> &mut usize {
        &mut self.net_hdr_size
    }

    #[inline]
    /// Returns the size of the transport header
    ///
    /// This has to be set by the transport layer handler
    pub fn tspt_hdr_size(&self) -> usize {
        self.tspt_hdr_size
    }

    #[inline]
    /// Returns a mutable reference to the size of the transport header
    pub unsafe fn tspt_hdr_size_mut(&mut self) -> &mut usize {
        &mut self.tspt_hdr_size
    }

    /// Returns the size of the payload.
    pub fn payload_size(&self) -> usize {
        self.size - self.link_hdr_size() - self.net_hdr_size() -
        self.tspt_hdr_size()
    }

    #[inline]
    fn header_get<T>(&self, offset: usize) -> Option<&T> {
        if self.size < offset + mem::size_of::<T>() {
            return None;
        }

        unsafe {
            Some(& *(self.data.offset(offset as isize) as *const T))
        }
    }

    /// Get a reference to the link header.
    ///
    /// This will return None if the packet size is smaller than the size
    /// of the link header.
    pub fn link_header<T>(&self) -> Option<&T> {
        self.header_get(0)
    }

    /// Get a reference to the network header.
    ///
    /// This will return None if the packet size is smaller than the size
    /// of the network header.
    pub fn net_header<T>(&self) -> Option<&T> {
        self.header_get(self.link_hdr_size)
    }

    /// Get a reference to the transport header.
    ///
    /// This will return None if the packet size is smaller than the size
    /// of the transport header.
    pub fn tspt_header<T>(&self) -> Option<&T> {
        self.header_get(self.link_hdr_size() + self.net_hdr_size())
    }

    /// Extract the payload from the packet.
    ///
    /// This will return None if there is no payload
    pub fn payload(&self) -> Option<&[u8]> {
        let offset = self.link_hdr_size() + self.net_hdr_size() +
                     self.tspt_hdr_size();

        if self.size <= offset {
            return None;
        }

        unsafe {
            Some(slice::from_raw_parts(self.data.offset(offset as isize),
                                       self.payload_size()))
        }
    }
}

impl Drop for Packet {
    fn drop(&mut self) {
        __rust_deallocate(self.page, PAGE_SIZE, PAGE_SIZE);
    }
}
