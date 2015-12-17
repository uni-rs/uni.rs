# Code organisation

The code of the unikernel is separated in different crates/libraries.

- rust-libs: Rust internal libraries
- deps: External dependencies
- libheap: Crate that contains implementation of different allocators.
- liballoc_uni: Wrapper that provides an allocator to Uni.rs
- libintrusive: Implementation of various intrusive containers
- libuni: Main crate that exports all Uni.rs's API.
- libboot: This library is responsible to wrap the user's application and
correctly initialize and setup the environment and the platform. It is not
directly accessible from the user's application.
