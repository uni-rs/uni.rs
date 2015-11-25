#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use self::x86::*;

mod x86 {
    pub fn wmb() {
        unsafe {
            asm!("sfence" ::: "memory" : "volatile");
        }
    }
}
