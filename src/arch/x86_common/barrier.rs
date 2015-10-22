pub fn mb() {
    unsafe {
        asm!("mfence" ::: "memory" : "volatile");
    }
}

pub fn rmb() {
    unsafe {
        asm!("lfence" ::: "memory" : "volatile");
    }
}

pub fn wmb() {
    unsafe {
        asm!("sfence" ::: "memory" : "volatile");
    }
}
