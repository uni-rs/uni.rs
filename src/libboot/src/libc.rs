#[no_mangle]
pub unsafe extern "C" fn memset(ptr: *mut u8, val: i32, num: usize) -> *mut u8 {
    let mut i = 0;

    while i < num {
        *ptr.offset(i as isize) = val as u8;

        i += 1;
    }

    ptr
}

#[no_mangle]
pub unsafe extern fn memcpy(dest: *mut u8, src: *const u8,
                            n: usize) -> *mut u8 {
    let mut i = 0;

    while i < n {
        *dest.offset(i as isize) = *src.offset(i as isize);
        i += 1;
    }

    dest
}

#[no_mangle]
pub unsafe extern fn memmove(dest: *mut u8, src: *const u8,
                             mut n: usize) -> *mut u8 {
    if dest as *const u8 <= src {
        return memcpy(dest, src, n);
    }

    while n != 0 {
        n -= 1;
        *dest.offset(n as isize) = *src.offset(n as isize);
    }

    dest
}
