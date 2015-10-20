extern crate gcc;

use std::fs;
use std::env;
use std::path::Path;

fn get_target() -> Option<&'static str> {
    let env_target = env::var("TARGET").unwrap();
    let v: Vec<&str> = env_target.split('-').collect();

    match *v.first().unwrap() {
        "i686" => Some("x86"),
        "x86_64" => Some("x86_64"),
        _ => None,
    }
}

fn build_libuniarch(arch_path: &std::path::PathBuf) {
    let arch_content = fs::read_dir(arch_path).unwrap();
    let mut gcc_config = gcc::Config::new();

    for p in arch_content {
        let path = p.unwrap().path();
        let extension = path.extension().unwrap().to_str().unwrap();

        if extension == "S" {
            gcc_config.file(path.to_str().unwrap().clone());
        }
    }

    gcc_config.compile("libuniarch.a");
}

fn main() {
    let target = match get_target() {
        Some(t) => t,
        None => panic!("Unable to determine the target arch"),
    };
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir_path = Path::new(&out_dir[..]);
    let arch_path = Path::new("src/arch/").join(target);

    fs::copy(arch_path.join("linker.ld"),
             out_dir_path.join("linker.ld")).unwrap();

    build_libuniarch(&arch_path);
}
