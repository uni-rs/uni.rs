#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use self::x86::*;

mod x86 {
    pub unsafe fn atomic_set_bit<T>(nr: usize, addr: *mut T) {
        asm!("lock bts $1, $0"
             :: "=*m" (addr as *mut u32), "Ir" (nr)
             : "memory"
             : "volatile");
    }

    pub unsafe fn atomic_clear_bit<T>(nr: usize, addr: *mut T) {
        asm!("lock btr $1, $0"
             :: "=*m" (addr as *mut u32), "Ir" (nr)
             : "memory"
             : "volatile");
    }

    pub fn wmb() {
        unsafe {
            asm!("sfence" ::: "memory" : "volatile");
        }
    }

    #[test]
    pub fn test_set_and_clear() {
        let mut array = [0u32; 4];

        unsafe  {
            atomic_set_bit(1, &mut array[0] as *mut u32);
            atomic_set_bit(2, &mut array[0] as *mut u32);
            atomic_set_bit(3, &mut array[0] as *mut u32);
            atomic_set_bit(32, &mut array[0] as *mut u32);
            atomic_set_bit(33, &mut array[0] as *mut u32);
            atomic_set_bit(34, &mut array[0] as *mut u32);
            atomic_set_bit(127, &mut array[0] as *mut u32);

            assert_eq!(array[0], 0xE);
            assert_eq!(array[1], 0x7);
            assert_eq!(array[2], 0x0);
            assert_eq!(array[3], 0x80000000);

            atomic_clear_bit(1, &mut array[0] as *mut u32);
            atomic_clear_bit(2, &mut array[0] as *mut u32);
            atomic_clear_bit(3, &mut array[0] as *mut u32);
            atomic_clear_bit(32, &mut array[0] as *mut u32);
            atomic_clear_bit(33, &mut array[0] as *mut u32);
            atomic_clear_bit(34, &mut array[0] as *mut u32);
            atomic_clear_bit(127, &mut array[0] as *mut u32);

            assert_eq!(array[0], 0x0);
            assert_eq!(array[1], 0x0);
            assert_eq!(array[2], 0x0);
            assert_eq!(array[3], 0x0);
        }
    }
}
