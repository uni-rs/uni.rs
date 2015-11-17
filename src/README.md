# Code organisation

The code of the unikernel is separated in different crates/libraries.

- libheap: Crate that contains implementation of different allocators.
- libxen: Crate that contains helpers and algorithms that ease development
on the xen platform
- libuni: Main crate that exports all Uni.rs's API.
- libboot: This library is responsible to wrap the user's application and
correctly initialize and setup the environment and the platform. It is not
directly accessible from the user's application.
