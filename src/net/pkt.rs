//! Network packet utility

use core::{mem, slice};

use sync::Arc;

use net::{Interface, InterfaceWeak};

use alloc_uni::{__rust_allocate, __rust_deallocate};

use hal::arch::defs::PAGE_SIZE;

/// Used to format a packet at the link or network layer
pub trait Formatter {
    /// Format the packet
    fn format(&self, builder: &mut Builder, intf: &Interface) -> Result<(), ()>;
}

/// Wrap packet creation
///
/// This object is convenient to create a new packet. It wraps allocation,
/// buffer management and final format of the packet. The packet yielded by
/// the `finalize` method can be directly sent out on the network.
pub struct Builder {
    /// Base pointer of the allocated page
    page: *mut u8,
    /// Pointer to the beginning of the data
    data: *mut u8,
    /// Size of the data
    size: usize,
    /// Formatter for the link layer
    link_fmt: Option<Arc<Formatter>>,
    /// Formatter for the network layer
    net_fmt: Option<Arc<Formatter>>,
    /// Was the packet generated yet ? Used in the destructor to determine if
    /// deallocation is necessary
    finalized: bool,
}

impl Builder {
    /// Create a new packet builder
    ///
    /// This does an allocation under the hood, which is why this method can
    /// fail.
    pub fn new() -> Result<Self, ()> {
        let page = __rust_allocate(PAGE_SIZE, PAGE_SIZE);

        if page.is_null() {
            Err(())
        } else {
            Ok(Builder {
                page: page,
                data: unsafe { page.offset(PAGE_SIZE as isize) },
                size: 0,
                link_fmt: None,
                net_fmt: None,
                finalized: false,
            })
        }
    }

    #[inline]
    /// Returns the current size of the packet
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline]
    /// Get the data contained in the packet as a slice
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.data, self.size)
        }
    }

    #[inline]
    /// Set the formatter for the link layer
    pub fn set_link_fmt(&mut self, fmt: Arc<Formatter>) {
        self.link_fmt = Some(fmt);
    }

    #[inline]
    /// Set the formatter for the network layer
    pub fn set_net_fmt(&mut self, fmt: Arc<Formatter>) {
        self.net_fmt = Some(fmt);
    }

    /// Write data inside the packet
    pub fn write(&mut self, data: &[u8]) -> Result<(), ()> {
        // Verify that we won't overflow the buffer
        if self.size + data.len() > PAGE_SIZE {
            return Err(());
        }

        self.size += data.len();

        unsafe {
            self.data = self.data.offset(- (data.len() as isize));
        }

        // Get the internal buffer as a slice to copy the data
        let self_data_slice = unsafe {
            slice::from_raw_parts_mut(self.data, data.len())
        };

        // Copy the data
        self_data_slice.clone_from_slice(data);

        Ok(())
    }

    /// Generate the packet
    pub fn finalize(mut self, intf: &Interface) -> Result<Packet, ()> {
        if let Some(link_fmt) = self.link_fmt.clone() {
            if let Some(net_fmt) = self.net_fmt.clone() {
                try!(net_fmt.format(&mut self, intf));
            }

            try!(link_fmt.format(&mut self, intf));
        } else {
            return Err(());
        }

        let offset = self.data as usize - self.page as usize;
        let pkt = unsafe {
            Packet::new(self.page, offset, self.size)
        };

        self.finalized = true;

        Ok(pkt)
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        if !self.finalized {
            __rust_deallocate(self.page, PAGE_SIZE, PAGE_SIZE);
        }
    }
}

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
