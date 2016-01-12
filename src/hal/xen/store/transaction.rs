use core::mem;

use core::str::FromStr;

use vec::Vec;
use string::String;
use string::ToString;

use ffi::CString;

use cell::GlobalCellMutRef;

use hal::xen::store::{Result, Error, XenbusState};

use super::imp::{XenStoreImpl, RequestBuilder, XsdSockmsgType};

/// A transaction with the Xen Store
///
/// Note: the transaction is automatically ended on `Drop` if not already
/// down.
pub struct Transaction<'a> {
    ended: bool,
    tx_id: u32,
    imp: GlobalCellMutRef<'a, XenStoreImpl>,
}

impl<'a> Transaction<'a> {
    #[doc(hidden)]
    pub fn new(mut imp: GlobalCellMutRef<'a, XenStoreImpl>) -> Result<Transaction<'a>> {
        let empty_data = CString::new("").unwrap();

        let tx_req = {
            RequestBuilder::new(0).set_msg_type(XsdSockmsgType::TransactionStart)
                                  .append_data(empty_data.as_bytes_with_nul())
        };

        imp.send(tx_req).and_then(|data| {
            match String::from_utf8(data) {
                Err(..) => Err(Error::Conversion),
                Ok(s) => match u32::from_str(&s[..]) {
                    Err(..) => Err(Error::Conversion),
                    Ok(id) => Ok(Transaction {
                        ended: false,
                        tx_id: id,
                        imp: imp,
                    }),
                },
            }
        })
    }

    /// List a directory in the Xen Store
    pub fn directory_list(&mut self, directory: CString) -> Result<Vec<String>> {
        let req = {
            RequestBuilder::new(self.tx_id).set_msg_type(XsdSockmsgType::Directory)
                                           .append_data(directory.as_bytes_with_nul())
        };

        self.imp.send(req).and_then(|data| {
            let mut v = Vec::<String>::new();
            let mut s = String::new();

            for b in data {
                if b == 0 {
                    v.push(s);
                    s = String::new();
                } else {
                    s.push(b as char);
                }
            }

            Ok(v)
        })
    }

    /// Switch the state of the XenBus
    pub fn switch_state(&mut self, path: CString,
                        state: XenbusState) -> Result<()> {
        let old_state: XenbusState = {
            let old_state_string = try!(self.read(path.clone()));
            let old_state_result = u8::from_str(old_state_string.as_str());

            unsafe {
                mem::transmute(
                    try!(old_state_result.map_err(|_| Error::Conversion))
                )
            }
        };

        if old_state != state {
            let s: u8 = unsafe { mem::transmute(state) };
            let state_cstring = CString::new(s.to_string());

            self.write(path,
                       try!(state_cstring.map_err(|_| Error::Conversion)))
        } else {
            Ok(())
        }
    }

    /// Read the value pointed by `key`
    pub fn read(&mut self, key: CString) -> Result<String> {
        let req = RequestBuilder::new(self.tx_id).set_msg_type(XsdSockmsgType::Read)
                                                 .append_data(key.as_bytes_with_nul());

        self.imp.send(req).and_then(|data| {
            match String::from_utf8(data) {
                Err(..) => Err(Error::Conversion),
                Ok(s) => Ok(s),
            }
        })
    }

    /// Write the value of `key`
    pub fn write(&mut self, key: CString, value: CString) -> Result<()> {
        let req = {
            RequestBuilder::new(self.tx_id).set_msg_type(XsdSockmsgType::Write)
                                           .append_data(key.as_bytes_with_nul())
                                           .append_data(value.as_bytes())
        };

        self.imp.send(req).map(|_| ())
    }

    fn _end(&mut self, data: CString) -> Result<()> {
        let req = {
            RequestBuilder::new(self.tx_id).set_msg_type(XsdSockmsgType::TransactionEnd)
                                           .append_data(data.as_bytes_with_nul())
        };

        self.imp.send(req).and_then(|data| {
            match String::from_utf8(data) {
                Err(..) => Err(Error::Conversion),
                Ok(s) => {
                    match s.as_ref() {
                        "OK" => {
                            self.ended = true;
                            Ok(())
                        }
                        _ => Err(Error::Unknown),
                    }
                }
            }
        })
    }

    /// End the transaction
    pub fn end(&mut self) -> Result<()> {
        self._end(CString::new("T").unwrap())
    }

    /// Abort the transaction
    pub fn abort(&mut self) -> Result<()> {
        self._end(CString::new("F").unwrap())
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if !self.ended {
            self.abort().unwrap();
        }
    }
}
