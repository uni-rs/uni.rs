use xen::defs::{StartInfo, SharedInfo, ConsoleInterface};

pub mod defs;
pub mod memory;

pub use self::memory::init as init_memory;

extern {
    // Start info is not present on all architecture, this is why this
    // was made a global variable only for x86_*
    pub static start_info: *const StartInfo;
}

// XXX: Move somewhere else
extern {
    pub static _shared_info: SharedInfo;
}

pub fn init() {
    unsafe {
        let console_vaddr: memory::page::Vaddr;

        memory::map_shared_info();

        console_vaddr = memory::page::mfn_to_vaddr((*start_info).domu_console.mfn);

        ::console::init(console_vaddr as *mut ConsoleInterface,
                        (*start_info).domu_console.evtchn);
    }
}
