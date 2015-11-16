use heap::FirstFit;

use os::lock::SpinLock;

static mut ALLOCATOR: SpinLock<Option<FirstFit>> = SpinLock::new(None);

pub unsafe fn init(region_start: usize, region_size: usize) {
    let mut guard = ALLOCATOR.lock();

    *guard = Some(FirstFit::new(region_start as *mut u8, region_size));
}
