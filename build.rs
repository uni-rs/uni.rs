extern crate gcc;

use std::fs;
use std::env;
use std::path::Path;

fn target_arch_get() -> &'static str {
    let target = env::var("TARGET").unwrap();

    if target.starts_with("x86_64") {
        "x86_64"
    } else if target.starts_with("i686") {
        "i686"
    } else {
        panic!("Unsupported architecture: {}", target);
    }
}

fn build_lib_boot() {
    let arch = target_arch_get();

    let src_dir = Path::new("src/hal/xen/boot/arch/").join(arch);

    gcc::compile_library("libboot.a",
                         &[src_dir.join("boot.S").to_str().unwrap()]);
}

fn build_lib_switch() {
    let arch = target_arch_get();

    let src_dir = Path::new("src/thread/context").join(arch);

    gcc::compile_library("libswitch.a",
                         &[src_dir.join("switch.S").to_str().unwrap()]);
}

fn copy_linker_script(out_path: &str) {
    let arch = target_arch_get();
    let out_dir = Path::new(&out_path);
    let linker_dir = Path::new("src/hal/xen/boot/arch/").join(arch);

    fs::copy(linker_dir.join("linker.ld"), out_dir.join("linker.ld")).unwrap();
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    build_lib_boot();
    build_lib_switch();

    copy_linker_script(&out_dir);
}
