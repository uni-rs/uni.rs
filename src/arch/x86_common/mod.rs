use ::xen::StartInfo;

use ::xen::console::console;
use ::xen::console::ConsoleInterface;

pub mod defs;
pub mod barrier;
pub mod memory;

extern {
    // Start info is not present on all architecture, this is why this
    // was made a global variable only for x86_*
    pub static start_info: *const StartInfo;
}

pub fn init() {
    unsafe {
        let console_vaddr: memory::page::Vaddr;

        memory::map_shared_info();

        console_vaddr = memory::page::mfn_to_vaddr((*start_info).domu_console.mfn);

        console().set_port((*start_info).domu_console.evtchn);
        console().set_interface(console_vaddr as *mut ConsoleInterface);
    }
}
