//! Implementation of the Xenstore protocol

use core::result;
use core::str::FromStr;

use ffi::CString;

use cell::GlobalCell;

use self::imp::XenStoreImpl;

use hal::xen::defs::{XenstoreInterface, EvtchnPort};

pub use self::transaction::Transaction;

static STORE: GlobalCell<XenStoreImpl> = GlobalCell::new();

mod imp;
mod transaction;

/// Result of an operation within the Xen Store
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
/// Error code that can be returned by a Xen Store operation
pub enum Error {
    /// Unknown error happened
    Unknown,
    /// A type conversion error
    Conversion,
    /// Invalid argument
    Inval,
    /// Permission denied
    Acces,
    /// File exists
    Exist,
    /// Is a directory
    Isdir,
    /// No such file or directory
    Noent,
    /// Out of memory
    Nomem,
    /// No space left on device
    Nospc,
    /// I/O error
    Io,
    /// Directory not empty
    Notempty,
    /// Function not implemented
    Nosys,
    /// Read-only filesystem
    Rofs,
    /// Device or resource is busy
    Busy,
    /// Try again
    Again,
    /// Transport endpoint is already connected
    Isconn,
}

/// Main interface of the Xen Store
pub struct XenStore;

impl XenStore {
    #[doc(hidden)]
    pub unsafe fn init(interface: *mut XenstoreInterface, port: EvtchnPort) {
        STORE.set(XenStoreImpl::new(interface, port));
    }

    #[doc(hidden)]
    pub unsafe fn init_event() {
        STORE.as_mut().init_event();
    }

    /// Start a new transaction within the Xen Store
    pub fn start_transaction<'a>() -> Result<Transaction<'a>> {
        Transaction::new(STORE.as_mut())
    }

    /// Read a value from the xen store and convert it
    pub fn read_value<T: FromStr>(path: CString) -> Result<T> {
        let mut t = try!(Self::start_transaction());

        let val = try!(t.read(path));

        T::from_str(val.as_str()).or_else(|_| Err(Error::Conversion))
    }
}
