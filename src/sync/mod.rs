//! Synchronisation primitives for Uni.rs

#[cfg(not(test))]
pub use alloc::arc::Arc;

pub mod spin;
