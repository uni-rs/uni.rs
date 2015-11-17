#[no_mangle]
pub unsafe extern "C" fn memset(ptr: *mut u8, val: i32, num: usize) -> *mut u8 {
    let mut i = 0;

    while i < num {
        *ptr.offset(i as isize) = val as u8;

        i += 1;
    }

    ptr
}
