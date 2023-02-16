use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    if !Path::new("./liblkl.so").exists() {
        Command::new("git")
            .args(&["clone", "https://github.com/docfate111/linux-lkl.git", "linux"])
            .current_dir(&Path::new(&project_dir))
            .status()
            .unwrap();
        Command::new("make")
            .args(&["-C", "linux/tools/lkl", "ARCH=lkl", "-j16"])
            .current_dir(&Path::new(&project_dir))
            .status()
            .unwrap();
        Command::new("cp")
            .args(&["linux/tools/lkl/lib/liblkl.so", "."])
            .current_dir(&Path::new(&project_dir))
            .status()
            .unwrap();
    }
    println!("cargo:rustc-link-search=native=linux/tools/lkl");
    //println!("cargo:rustc-link-search={}", project_dir); // the "-L" flag
    println!("cargo:rustc-link-lib=lkl"); // the "-l" flag
    // println!("cargo:rustc-env=LD_LIBRARY_PATH=.");*/
}
