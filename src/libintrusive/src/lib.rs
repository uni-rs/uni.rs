//! Implementation of various intrusive containers:
//! - Doubly Linked List
//! - Queue

#![feature(no_std)]
#![feature(unique)]
#![feature(const_fn)]
#![no_std]

#[cfg(test)]
extern crate std;

pub mod link;

pub mod list;
pub mod queue;
