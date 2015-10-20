#[cfg(target_arch = "x86")]
#[path="x86"]
mod imp {
    pub mod types;
}

pub use self::imp::types;
