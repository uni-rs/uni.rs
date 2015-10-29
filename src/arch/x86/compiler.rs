#[no_mangle]
pub extern "C" fn __udivdi3(num: u64, den: u64) -> u64 {
    let (ret, _) = __udivmoddi(num, den);

    ret
}

#[no_mangle]
pub extern "C" fn __umoddi3(num: u64, den: u64) -> u64 {
    let (_, ret) = __udivmoddi(num, den);

    ret
}

fn __udivmoddi(mut num: u64, mut den: u64) -> (u64, u64) {
    let mut quot = 0u64;
    let mut qbit = 1u64;

    if den == 0 {
        panic!("Division by 0");
    }

    while den as i64 >= 0 {
        den <<= 1;
        qbit <<= 1;
    }

    while qbit != 0 {
        if den <= num {
            num -= den;
            quot += qbit;
        }

        den >>= 1;
        qbit >>= 1;
    }

    (quot, num)
}
