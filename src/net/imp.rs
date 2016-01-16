use cell::GlobalCell;

pub static STACK: GlobalCell<StackImpl> = GlobalCell::new();

/// Uni.rs network stack
pub struct Stack;

impl Stack {
    #[doc(hidden)]
    pub fn init() {
        STACK.set(StackImpl::new());
    }
}

pub struct StackImpl;

unsafe impl Sync for StackImpl {}

impl StackImpl {
    pub fn new() -> Self {
        StackImpl
    }
}
