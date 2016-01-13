//! Implementation of Xen's grant table

use core::mem;
use core::ptr;
use core::intrinsics::atomic_cxchg;

use vec::Vec;
use vec_deque::VecDeque;

use string::String;
use string::ToString;

use alloc_uni::__rust_allocate;

use cell::GlobalCell;

use sync::spin::InterruptSpinLock;

use hal::mmu::{Vaddr, Mfn};

use hal::arch::utils::wmb;
use hal::arch::defs::PAGE_SIZE;

use hal::xen::defs::{Ulong, DOMID_SELF};

use hal::xen::arch::x86::memory::map_non_contiguous_mfn;

static TABLE: GlobalCell<TableImpl> = GlobalCell::new();

#[repr(i16)]
#[allow(dead_code)]
#[derive(PartialEq, Debug)]
/// Error status that can be returned by the table initialization
pub enum GntStatus {
    Ok = 0,
    GeneralError = -1,
    BadDomain = -2,
    BadGntref = -3,
    BadHandle = -4,
    BadVirtAddr = -5,
    BadDevAddr = -6,
    NoDeviceSpace = -7,
    PermissionDenied = -8,
    BadPage = -9,
    BadCopyArg = -10,
    AddressTooBig = -11,
    Eagain = -12,
}

#[allow(dead_code)]
enum GnttabOp {
     MapGrantRef = 0,
     UnmapGrantRef = 1,
     SetupTable = 2,
     DumpTable = 3,
     Transfer = 4,
     Copy = 5,
     QuerySize = 6,
     UnmapAndReplace = 7,
     SetVersion = 8,
     GetStatusFrames = 9,
     GetVersion = 10,
     CacheFlush = 12,
}

#[repr(C)]
struct GnttabSetupTable {
    dom: u16,
    nr_frames: u32,
    status: GntStatus,
    frame_list: *mut Mfn,
}

#[repr(C)]
struct GnttabQuerySize {
    dom: u16,
    nr_frames: u32,
    max_nr_frames: u32,
    status: GntStatus,
}

#[allow(dead_code)]
const GTF_INVALID: u16 = 0 << 0;
#[allow(dead_code)]
const GTF_PERMIT_ACCESS: u16 = 1 << 0;
#[allow(dead_code)]
const GTF_ACCEPT_TRANSFER: u16 = 2 << 0;
#[allow(dead_code)]
const GTF_TYPE_MASK: u16 = 3 << 0;

#[allow(dead_code)]
const GTF_READONLY: u16 = 1 << 2;
#[allow(dead_code)]
const GTF_READING: u16 = 1 << 3;
#[allow(dead_code)]
const GTF_WRITING: u16 = 1 << 4;

#[allow(dead_code)]
const GTF_TRANSFER_COMMITTED: u16 = 1 << 2;
#[allow(dead_code)]
const GTF_TRANSFER_COMPLETED: u16 = 1 << 3;

/// The global grant table wrapper
pub struct Table;

impl Table {
    #[doc(hidden)]
    /// Initialize Xen's grant table
    pub fn init() -> Result<(), GntStatus> {
        let imp = try!(TableImpl::new());

        TABLE.set(imp);

        Ok(())
    }

    /// Allocate a new entry in the Grant table
    pub fn alloc_ref() -> Option<Ref> {
        TABLE.as_mut().alloc_ref()
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
// XXX: What about ending access/freeing the grant on Drop. One problem atm:
// it's clonable.
/// A grant reference index inside the grant table
pub struct Ref(u32);

impl Ref {
    /// Grant access to a page to the domain with id `domid`
    ///
    /// Note that the page has to be expressed as a Machine Frame Number
    /// (`mfn`)
    pub fn grant_access(&self, domid: u16, mfn: Mfn, readonly: bool) {
        let entry = unsafe {
            &mut *TABLE.as_mut().grant_table.offset(self.0 as isize)
        };

        entry.frame = *mfn as u32;
        entry.domid = domid;
        wmb();
        entry.flags = GTF_PERMIT_ACCESS | if readonly { GTF_READONLY } else { 0 };
    }

    /// End a previously granted access
    pub fn end_access(&mut self) {
        let mut entry = unsafe {
            &mut *TABLE.as_mut().grant_table.offset(self.0 as isize)
        };

        let mut flags = entry.flags;

        loop {
            let nflags = unsafe { atomic_cxchg(&mut entry.flags, flags, 0) };

            if nflags == 0 {
                break;
            }

            flags = nflags;
        }
    }
}

impl ToString for Ref {
    #[inline]
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

const RESERVED_ENTRIES: usize = 8;

#[repr(C)]
struct Entry {
    flags: u16,
    domid: u16,
    frame: u32,
}

struct TableImpl {
    grant_table: *mut Entry,
    free: InterruptSpinLock<VecDeque<Ref>>,
}

// This is safe for 2 reasons:
// - free is protected by a spinlock
// - grant_table is only accessed via a Ref index. A ref is not Sync nor Send
//   and is unique. So a cell of the grant table can only be accessed from one
//   point in the code at the same time.
unsafe impl Sync for TableImpl {}

impl TableImpl {
    pub fn new() -> Result<Self, GntStatus> {
        let ret;
        let frame_count = Self::get_max_frame_count();
        let mut frames: Vec<Mfn> = vec![Mfn::new(0); frame_count];
        let mut setup = GnttabSetupTable {
            dom: DOMID_SELF,
            nr_frames: frame_count as u32,
            status: GntStatus::Ok,
            frame_list: frames.as_mut_slice().as_mut_ptr(),
        };

        // Get the frame list for the grant table from Xen
        unsafe {
            ret = Self::grant_table_op(GnttabOp::SetupTable, &mut setup, 1);
        }

        if ret < 0 {
            return Err(GntStatus::GeneralError)
        }

        if setup.status != GntStatus::Ok {
            return Err(setup.status)
        }

        // Allocate enough virtual space to map the grant table
        //
        // XXX: There is room for improvement here as this will remove
        // some pages that could be used as heap memory. When we support
        // allocating extra virtual memory, this should be used rather than
        // using heap space.
        let addr: *mut u8 = __rust_allocate(PAGE_SIZE * frame_count, PAGE_SIZE);

        if addr == ptr::null_mut() {
            return Err(GntStatus::GeneralError)
        }

        let vaddr = Vaddr::new(addr as usize);

        unsafe {
            // Map the grant table
            match map_non_contiguous_mfn(vaddr, frames.as_slice()) {
                Err(..) => Err(GntStatus::GeneralError),
                Ok(..) => {
                    let count = Self::get_grant_entries_count();
                    let table = TableImpl {
                        grant_table: addr as *mut _,
                        free: InterruptSpinLock::new(VecDeque::with_capacity(count)),
                    };

                    // Initialize free entries queue
                    for i in RESERVED_ENTRIES..count {
                        table.free.lock().push_back(Ref(i as u32));
                    }

                    Ok(table)
                }
            }
        }
    }

    pub fn alloc_ref(&mut self) -> Option<Ref> {
        self.free.lock().pop_front()
    }

    unsafe fn grant_table_op<T>(op: GnttabOp, uop: *mut T, count: u32) -> i32 {
        use hal::xen::hypercall::{hypercall3, HypercallKind};

        hypercall3(HypercallKind::GrantTableOp, op as Ulong,
                   uop as *mut u8 as Ulong, count as Ulong) as i32
    }

    /// Returns the maximum number of frames the grant table can be composed of
    fn get_max_frame_count() -> usize {
        let ret;
        let mut query = GnttabQuerySize {
            dom: DOMID_SELF,
            nr_frames: 0,
            max_nr_frames: 0,
            status: GntStatus::Ok,
        };

        unsafe {
            ret = Self::grant_table_op(GnttabOp::QuerySize, &mut query, 1);
        }

        if ret < 0 || query.status != GntStatus::Ok {
            // Legacy value
            return 4;
        }

        query.max_nr_frames as usize
    }

    /// Returns the maximum number of entries the grant table can contain
    fn get_grant_entries_count() -> usize {
        Self::get_max_frame_count() * PAGE_SIZE / mem::size_of::<Entry>()
    }
}
