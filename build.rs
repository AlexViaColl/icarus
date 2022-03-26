use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rustc-link-search=.");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=compile.sh");
    println!("cargo:rerun-if-changed=assets/shaders/shader.vert");
    println!("cargo:rerun-if-changed=assets/shaders/shader.frag");

    if !Path::new("stb_image.h").exists() {
        Command::new("/usr/bin/wget")
            .arg("https://raw.githubusercontent.com/nothings/stb/master/stb_image.h")
            .status()
            .unwrap();
    }

    if !Path::new("assets/models/viking_room.obj").exists() {
        Command::new("/usr/bin/wget")
            .arg("-O")
            .arg("assets/models/viking_room.obj")
            .arg("https://vulkan-tutorial.com/resources/viking_room.obj")
            .status()
            .unwrap();

        Command::new("/usr/bin/wget")
            .arg("-O")
            .arg("assets/textures/viking_room.png")
            .arg("https://vulkan-tutorial.com/resources/viking_room.png")
            .status()
            .unwrap();

        Command::new("/usr/bin/wget")
            .arg("-O")
            .arg("assets/textures/texture.jpg")
            .arg("https://vulkan-tutorial.com/images/texture.jpg")
            .status()
            .unwrap();
    }

    Command::new("/bin/sh").arg("compile.sh").status().unwrap();
}
