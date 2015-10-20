#[cfg(target_arch = "x86")]
#[path="x86"]
mod imp {
    pub mod types;
}

#[cfg(target_arch = "x86_64")]
#[path="x86_64"]
mod imp {
    pub mod types;
}

pub use self::imp::types;
