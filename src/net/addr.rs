use fmt;

const COUNT_HWADDR_BYTES: usize = 6;

#[repr(C)]
#[derive(Copy, Clone)]
/// A MAC address
pub struct HwAddr {
    bytes: [u8; COUNT_HWADDR_BYTES],
}

impl HwAddr {
    /// Create an empty hardware address (i.e., 00:00:00:00:00:00)
    pub fn empty() -> Self {
        HwAddr {
            bytes: [0; COUNT_HWADDR_BYTES],
        }
    }

    /// Create an hardware address from bytes.
    ///
    /// This method is unsafe because the slice *MUST* contain at least 6
    /// elements.
    pub unsafe fn from_bytes(bytes: &[u8]) -> Self {
        let mut ret = Self::empty();

        ret.bytes[0] = bytes[0];
        ret.bytes[1] = bytes[1];
        ret.bytes[2] = bytes[2];
        ret.bytes[3] = bytes[3];
        ret.bytes[4] = bytes[4];
        ret.bytes[5] = bytes[5];

        ret
    }

    fn format(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!("{:x}:{:x}:{:x}:{:x}:{:x}:{:x}", self.bytes[0],
                                 self.bytes[1], self.bytes[2], self.bytes[3],
                                 self.bytes[4], self.bytes[5]))
    }

    #[inline]
    /// Returns the internal representation of an hardware address
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..]
    }
}

impl PartialEq for HwAddr {
    fn eq(&self, rhs: &Self) -> bool {
        self.bytes[0] == rhs.bytes[0] &&
        self.bytes[1] == rhs.bytes[1] &&
        self.bytes[2] == rhs.bytes[2] &&
        self.bytes[3] == rhs.bytes[3] &&
        self.bytes[4] == rhs.bytes[4] &&
        self.bytes[5] == rhs.bytes[5]
    }
}

impl fmt::Debug for HwAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.format(f)
    }
}

impl fmt::Display for HwAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.format(f)
    }
}
