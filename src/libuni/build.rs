extern crate gcc;

use std::path::Path;
use std::env;

fn arch_get() -> &'static str {
    let target = env::var("TARGET").unwrap();

    if target.starts_with("x86_64") {
        "x86_64"
    } else if target.starts_with("i686") {
        "x86"
    } else {
        panic!("Unsupported architecture: {}", target);
    }
}

fn build_lib_switch() {
    let arch = arch_get();

    let src_dir = Path::new("src/thread/context").join(arch);

    gcc::compile_library("libswitch.a",
                         &[src_dir.join("switch.S").to_str().unwrap()]);
}

fn main() {
    build_lib_switch();
}
