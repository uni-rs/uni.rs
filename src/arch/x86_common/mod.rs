use ::xen::StartInfo;

pub mod defs;
pub mod barrier;
pub mod memory;

extern {
    // Start info is not present on all architecture, this is why this
    // was made a global variable only for x86_*
    pub static start_info: *const StartInfo;
}
