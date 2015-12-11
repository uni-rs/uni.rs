//! Definition of various functions and variables to handle x86 traps

use core::mem;

use xen::defs::{TrapInfo, FLAT_KERNEL_CS};
use xen::arch::x86::traps::set_trap_table;

use libc::memset;

extern "C" {
    fn division_error();
    fn debug();
    fn nmi();
    fn breakpoint();
    fn overflow();
    fn bound();
    fn invalid_opcode();
    fn device_not_available();
    fn double_fault();
    fn coproc_seg_overrun();
    fn invalid_tss();
    fn seg_not_present();
    fn stack_seg_fault();
    fn gpf();
    fn page_fault();
    fn fp_exception();
    fn align_check();
    fn machine_check();
    fn simd_exception();
}

const TRAPS_COUNT: usize = 20;

static mut TRAPS: [TrapInfo; TRAPS_COUNT] = [
    TrapInfo::new(0, 0, FLAT_KERNEL_CS, division_error),
    TrapInfo::new(1, 0, FLAT_KERNEL_CS, debug),
    TrapInfo::new(2, 0, FLAT_KERNEL_CS, nmi),
    TrapInfo::new(3, 0, FLAT_KERNEL_CS, breakpoint),
    TrapInfo::new(4, 0, FLAT_KERNEL_CS, overflow),
    TrapInfo::new(5, 0, FLAT_KERNEL_CS, bound),
    TrapInfo::new(6, 0, FLAT_KERNEL_CS, invalid_opcode),
    TrapInfo::new(7, 0, FLAT_KERNEL_CS, device_not_available),
    TrapInfo::new(8, 0, FLAT_KERNEL_CS, double_fault),
    TrapInfo::new(9, 0, FLAT_KERNEL_CS, coproc_seg_overrun),
    TrapInfo::new(10, 0, FLAT_KERNEL_CS, invalid_tss),
    TrapInfo::new(11, 0, FLAT_KERNEL_CS, seg_not_present),
    TrapInfo::new(12, 0, FLAT_KERNEL_CS, stack_seg_fault),
    TrapInfo::new(13, 0, FLAT_KERNEL_CS, gpf),
    TrapInfo::new(14, 0, FLAT_KERNEL_CS, page_fault),
    TrapInfo::new(16, 0, FLAT_KERNEL_CS, fp_exception),
    TrapInfo::new(17, 0, FLAT_KERNEL_CS, align_check),
    TrapInfo::new(18, 0, FLAT_KERNEL_CS, machine_check),
    TrapInfo::new(19, 0, FLAT_KERNEL_CS, simd_exception),

    // Fake entry, there is no way to pass a null pointer for a function
    // pointer in rust (not that I know of). Instead this part will be
    // memset before registering the trap table
    TrapInfo::new(0, 0, 0, division_error),
];

macro_rules! gen_trap {
    ( $name:ident, $num: expr, $short: expr, $long: expr ) => (
        #[no_mangle]
        pub extern "C" fn $name(_: *const u8, error_code: usize) {
            do_trap($num, $short, $long, error_code);
        }
    )
}

gen_trap!(do_division_error, 0, "DE", "Divide-by-zero Error");
gen_trap!(do_debug, 1, "DB", "Debug");
gen_trap!(do_nmi, 2, "NMI", "Non-Maskable Interrupt");
gen_trap!(do_breakpoint, 3, "BP", "Breakpoint");
gen_trap!(do_overflow, 4, "OF", "Overflow");
gen_trap!(do_bound, 5, "BR", "Bound Range Exceeded");
gen_trap!(do_invalid_opcode, 6, "UD", "Invalid Opcode");
gen_trap!(do_device_not_available, 7, "NM", "Device Not Available");
gen_trap!(do_double_fault, 8, "DF", "Double Fault");
gen_trap!(do_coproc_seg_overrun, 9, "", "Coprocessor Segment Overrun");
gen_trap!(do_invalid_tss, 10, "TS", "Invalid TSS");
gen_trap!(do_seg_not_present, 11, "NP", "Segment Non Present");
gen_trap!(do_stack_seg_fault, 12, "SS", "Stack-Segment Fault");
gen_trap!(do_gpf, 13, "GP", "General Protection Fault");
gen_trap!(do_page_fault, 14, "PF", "Page Fault");
gen_trap!(do_fp_exception, 16, "MF", "x87 Floating-Point Exception");
gen_trap!(do_align_check, 17, "AC", "Alignment Check");
gen_trap!(do_machine_check, 18, "MC", "Machine Check");
gen_trap!(do_simd_exception, 19, "XM/#XF", "SIMD Floating-Point Exception");

fn do_trap(num: u32, short: &str, long: &str, error_code: usize) {
    panic!("Unresolved trap {} (#{}): {}, error code: {}", num, short,
           long, error_code);
}

pub fn init() {
    unsafe {
        let null_entry = &mut TRAPS[TRAPS_COUNT - 1] as *mut TrapInfo;

        memset(null_entry as *mut u8, 0, mem::size_of::<TrapInfo>());

        let trap_begin = &TRAPS[0] as *const TrapInfo;

        set_trap_table(trap_begin);
    }

    println!("Uni.rs traps handlers installed");
}
