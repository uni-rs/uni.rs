//! Implementation of Xen's ring mechanism

use core::{mem, cmp};

use core::marker::PhantomData;

use rlibc::memset;

use alloc_uni::__rust_allocate;

use hal::mmu::{Vaddr, Mfn};

use hal::arch::utils::mb;

use hal::arch::defs::PAGE_SIZE;

use hal::xen::grant::Ref as GrantRef;
use hal::xen::grant::Table as GrantTable;

// Round a 32-bit unsigned constant down to the nearest power of two.
// Taken from xen/interface/io/ring.h
macro_rules! RD2 {
    ($x:expr) => (
        if $x & 0x2 != 0 {
            0x2
        } else {
            $x & 0x1
        }
    );
}

macro_rules! RD4 {
    ($x:expr) => (
        if $x & 0xC != 0 {
            RD2!($x >> 2) << 2
        } else {
            RD2!($x)
        }
    );
}

macro_rules! RD8 {
    ($x:expr) => (
        if $x & 0xF0 != 0 {
            RD4!($x >> 4) << 4
        } else {
            RD4!($x)
        }
    );
}

macro_rules! RD16 {
    ($x:expr) => (
        if $x & 0xFF00 != 0 {
            RD8!($x >> 8) << 8
        } else {
            RD8!($x)
        }
    );
}

macro_rules! RD32 {
    ($x:expr) => (
        if $x & 0xFFFF0000 != 0 {
            RD16!($x >> 16) << 16
        } else {
            RD16!($x)
        }
    );
}

/// Ring index type
pub type Idx = u32;

#[repr(C)]
/// Shared ring page
///
/// Equivalent of struct __name##_sring defined by DEFINE_RING_TYPES
struct XenSharedRing {
    req_prod: Idx,
    req_event: Idx,
    rsp_prod: Idx,
    rsp_event: Idx,
    pad: [u8; 48],
    ring: u8, // Variable length
}

// XXX: deallocate/end grant access on Drop ?
/// Xen's shared ring
///
/// `Req` is the type of the request
///
/// `Resp` is the type of the response
pub struct SharedRing<Req, Resp> {
    sring: *mut XenSharedRing,
    size: usize,
    elt_size: usize,
    grant_ref: Option<GrantRef>,
    _req: PhantomData<Req>,
    _resp: PhantomData<Resp>,
}

impl<Req, Resp> SharedRing<Req, Resp> {
    /// Allocate a new shared ring
    ///
    /// May return None for two reasons:
    /// - `size` is not aligned on PAGE_SIZE
    /// - The system is out of memory
    pub fn new(size: usize) -> Option<SharedRing<Req, Resp>> {
        if size < PAGE_SIZE || size % PAGE_SIZE != 0 {
            return None;
        }

        // Allocate memory for the ring
        let sring = __rust_allocate(size, PAGE_SIZE) as *mut XenSharedRing;

        if sring.is_null() {
            return None;
        }

        // Initialize the ring
        unsafe {
            memset(sring as *mut u8, 0, PAGE_SIZE);

            (*sring).req_prod = 0;
            (*sring).rsp_prod = 0;
            (*sring).req_event = 1;
            (*sring).rsp_event = 1;
        }

        Some(SharedRing {
            sring: sring,
            size: size,
            elt_size: cmp::max(mem::size_of::<Req>(), mem::size_of::<Resp>()),
            grant_ref: None,
            _req: PhantomData,
            _resp: PhantomData,
        })
    }

    /// Create a shared ring from an existing memory area
    pub unsafe fn from_ptr(ptr: *mut u8, size: usize) -> SharedRing<Req, Resp> {
        SharedRing {
            sring: ptr as *mut _,
            size: size,
            elt_size: cmp::max(mem::size_of::<Req>(), mem::size_of::<Resp>()),
            grant_ref: None,
            _req: PhantomData,
            _resp: PhantomData,
        }
    }

    #[inline]
    /// Returns the request production index
    pub fn req_prod(&self) -> Idx {
        unsafe {
            (*self.sring).req_prod
        }
    }

    #[inline]
    /// Returns the response production index
    pub fn rsp_prod(&self) -> Idx {
        unsafe {
            (*self.sring).rsp_prod
        }
    }

    #[inline]
    /// Returns the request event index
    pub fn req_event(&self) -> Idx {
        unsafe {
            (*self.sring).req_event
        }
    }

    #[inline]
    /// Returns the response event index
    pub fn rsp_event(&self) -> Idx {
        unsafe {
            (*self.sring).rsp_event
        }
    }

    /// Set the request production index
    ///
    /// This is unsafe as an invalid index can corrupt the ring
    pub unsafe fn req_prod_set(&mut self, req: Idx) {
        (*self.sring).req_prod = req;
        mb();
    }

    /// Set the response production index
    ///
    /// This is unsafe as an invalid index can corrupt the ring
    pub unsafe fn rsp_prod_set(&mut self, rsp: Idx) {
        (*self.sring).rsp_prod = rsp;
        mb();
    }

    #[inline]
    /// Set the request event index
    pub unsafe fn req_event_set(&mut self, req: Idx) {
        (*self.sring).req_event = req;
    }

    #[inline]
    /// Set the response event index
    pub unsafe fn rsp_event_set(&mut self, rsp: Idx) {
        (*self.sring).rsp_event = rsp;
    }

    /// Returns the size of the ring
    ///
    /// Equivalent to __RING_SIZE
    pub fn size(&self) -> usize {
        let ring_addr = unsafe { &(*self.sring).ring } as *const _ as isize;
        let sring_addr = self.sring as isize;

        RD32!((self.size as isize - ring_addr + sring_addr) as usize /
              self.elt_size)
    }

    /// Grant access to the ring to a foreign domain with id `domid`
    pub fn grant_access(&mut self, domid: u16) -> Option<GrantRef> {
        if let Some(ref r) = self.grant_ref {
            return Some(r.clone());
        }

        match GrantTable::alloc_ref() {
            None => None,
            Some(r) => {
                r.grant_access(domid, Mfn::from(Vaddr::from_ptr(self.sring)),
                               false);

                self.grant_ref = Some(r.clone());

                Some(r)
            }
        }
    }

    #[inline]
    fn ptr_from_index(&mut self, mut idx: usize) -> *mut u8 {
        idx = idx & (self.size() - 1);

        unsafe {
            let ring_ptr: *mut u8 = &mut (*self.sring).ring;

            ring_ptr.offset((idx * self.elt_size) as isize) as *mut u8
        }
    }

    /// Get a reference over the request at position `idx`
    pub fn request_from_index(&mut self, idx: usize) -> &mut Req {
        unsafe {
            &mut *(self.ptr_from_index(idx) as *mut Req)
        }
    }

    /// Get a reference over the response at position `idx`
    pub fn response_from_index(&mut self, idx: usize) -> &mut Resp {
        unsafe {
            &mut *(self.ptr_from_index(idx) as *mut Resp)
        }
    }
}
