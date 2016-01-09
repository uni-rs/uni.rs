//! Utilities related to FFI bindings.

use vec::Vec;
use boxed::Box;

#[derive(Clone)]
/// Represents an owned C style string
///
/// This type generates compatible C-strings from Rust. It guarantees that
/// there is no null bytes in the string and that the string is ended by a null
/// byte.
pub struct CString {
    data: Box<[u8]>,
}

impl CString {
    /// Creates a new C style string
    ///
    /// This will ensure that no null byte is present in the string or return
    /// and error if that's the case. It will also add the final null byte.
    pub fn new<T: Into<Vec<u8>>>(t: T) -> Result<CString, ()> {
        let mut v = t.into();

        for b in &v {
            if *b == 0 {
                return Err(());
            }
        }

        v.push(0);

        Ok(CString {
            data: v.into_boxed_slice(),
        })
    }

    /// Return the raw representation of the string without the trailing null
    /// byte
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.data.len() - 1]
    }

    /// Return the raw representation of the string.
    ///
    /// This buffer is guaranteed without intermediate null bytes and includes
    /// the trailing null byte.
    pub fn as_bytes_with_nul(&self) -> &[u8] {
        &*self.data
    }
}
