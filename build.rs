extern crate gcc;

use std::fs;
use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let prefix = Path::new(".");

    fs::copy(prefix.join("src/arch/x86/linker.ld"),
             out_dir.clone() + "/linker.ld").unwrap();

    gcc::compile_library("libuniarch.a", &["src/arch/x86/boot.S"]);

    println!("cargo:rustc-link-search=native={}", out_dir);
}
