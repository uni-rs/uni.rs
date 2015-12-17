pub use self::imp::*;

pub mod xen;

mod imp {
    use super::xen;

    #[inline]
    pub fn local_irq_disable() {
        xen::disable_upcalls();
    }

    #[inline]
    pub fn local_irq_enable() {
        xen::enable_upcalls();
    }

    #[inline]
    pub fn local_irq_save() -> usize {
        xen::disable_upcalls() as usize
    }

    #[inline]
    pub fn local_irq_restore(state: usize) {
        xen::set_upcalls_state(state as u8);
    }
}
