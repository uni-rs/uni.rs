#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use self::x86::*;

mod x86 {
    pub use core::intrinsics::atomic_xchg;

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

    pub fn first_bit(data: usize) -> usize {
        unsafe {
            let ret;

            asm!("bsf $1, $0\n\t\
                 jnz 1f\n\t
                 mov $$0, $0\n\t
                 1:\n\t"
                 : "=r" (ret)
                 : "r" (data)
                 :: "volatile");

            ret
        }
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

    #[test]
    pub fn test_first_bit() {
        assert_eq!(first_bit(0x0), 0);
        assert_eq!(first_bit(0x1), 0);
        assert_eq!(first_bit(0x2), 1);
        assert_eq!(first_bit(0x3), 0);
        assert_eq!(first_bit(0x8000), 15);
        assert_eq!(first_bit(0xFF80), 7);
        assert_eq!(first_bit(0x80000000), 31);
    }
}
