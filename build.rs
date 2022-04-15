use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rustc-link-search=.");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=compile.sh");
    println!("cargo:rerun-if-changed=assets/shaders/");

    if !Path::new("stb_image.h").exists() {
        Command::new("/usr/bin/wget")
            .arg("https://raw.githubusercontent.com/nothings/stb/master/stb_image.h")
            .status()
            .unwrap();
    }

    if !Path::new("glslc").exists() {
        Command::new("/usr/bin/wget")
            .arg("https://raw.githubusercontent.com/AlexViaColl/Icarus_deps/main/glslc.tar.gz")
            .status()
            .unwrap();

        Command::new("/usr/bin/tar").arg("-xzvf").arg("glslc.tar.gz").status().unwrap();
        std::fs::set_permissions("glslc", std::fs::Permissions::from_mode(0o770)).unwrap();
    }

    let output = Command::new("/bin/sh").arg("compile.sh").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "stderr: {}\nstdout: {}", stderr, stdout);

    assert!(Path::new("assets/shaders/simple.vert.spv").exists());
}
