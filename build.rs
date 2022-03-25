use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rustc-link-search=.");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=compile.sh");

    if !Path::new("stb_image.h").exists() {
        Command::new("/usr/bin/wget")
            .arg("https://raw.githubusercontent.com/nothings/stb/master/stb_image.h")
            .status()
            .unwrap();
    }

    Command::new("/bin/sh").arg("compile.sh").status().unwrap();
}
