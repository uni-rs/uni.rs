#[no_mangle]
pub unsafe fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;

    while i < n {
        *dst.offset(i as isize) = *src.offset(i as isize);
        i += 1;
    }

    dst
}

#[no_mangle]
pub unsafe fn memset(ptr: *mut u8, value: i32, n: usize) -> *mut u8 {
    let mut i = 0;

    while i < n {
        *ptr.offset(i as isize) = value as u8;
        i += 1;
    }

    ptr
}
