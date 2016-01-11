//! List the content of the root directory of the domain in the Xen store

#![feature(asm)]
#![feature(start)]
#![no_std]

#[macro_use]
extern crate uni;

use uni::ffi::CString;

use uni::hal::xen::store::XenStore;

#[start]
fn main(_: isize, _: *const *const u8) -> isize {
    let mut t = XenStore::start_transaction().unwrap();

    let domid = t.read(CString::new("domid").unwrap()).unwrap();

    let root = format!("/local/domain/{}", domid);

    println!("ls {} ->", root);

    let dirs = t.directory_list(CString::new(root).unwrap()).unwrap();

    for dir in &dirs {
        println!("  {}", dir);
    }

    t.end().unwrap();

    0
}
